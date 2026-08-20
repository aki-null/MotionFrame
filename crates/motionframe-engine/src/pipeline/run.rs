//! End-to-end pipeline: frames → atlas pair.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::io::tga::premultiply_alpha;
use crate::pipeline::analyze::{
    build_tail_flow_from_plan, clip_flow_linf, plan_batches, process_batch_slice_with_progress,
    update_max_strength,
};
use crate::pipeline::atlas::{blit_extrude, resize_nyquist};
use crate::pipeline::encode::{encode_r8g8_remap, encode_sidefx_labs};
use crate::pipeline::pack::{flat_pack, stagger_pack};
use crate::pipeline::{
    Flow, GenerateOptions, ImageF32, ImageRgba8, MotionVectorEncoding, PipelineError, Progress,
};
use rayon::prelude::*;

/// Result of the full pipeline: color atlas + motion atlas + metadata.
pub struct EncodeResult {
    /// Color atlas with all source frames blitted into a grid.
    pub color_atlas: ImageRgba8,
    /// Packed motion atlas (R8G8 encoded).
    pub motion_atlas: ImageRgba8,
    /// Max flow magnitude used for encoding normalization.
    pub strength: f64,
    /// Number of frames actually placed in the atlas.
    pub total_frames: u32,
    /// Atlas width in pixels.
    pub atlas_width: u32,
    /// Atlas height in pixels.
    pub atlas_height: u32,
    /// Number of tile columns in the atlas grid (tiles along X).
    pub columns: u32,
    /// Number of tile rows in the atlas grid (tiles along Y).
    pub rows: u32,
    /// Pack mode applied.
    pub pack_mode: PackMode,
    /// Whether the sequence loops.
    pub is_loop: bool,
    /// Whether the color atlas is stored with premultiplied alpha. Mirrors
    /// `GenerateOptions::premultiplied_alpha` so consumers (preview, metadata
    /// export) can interpret the atlas correctly without a back-channel.
    pub premultiplied_alpha: bool,
    /// Computed optical flow fields (for lazy visualization).
    pub flows: Vec<Flow>,
}

/// Atlas packing mode applied to the motion atlas.
///
/// Stagger halves texture fetches by interleaving two tiles into RGBA channels;
/// flat is the simpler fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackMode {
    /// Stagger pack: interleave pairs into 4 channels.
    Staggered,
    /// Flat pack: simple channel remapping.
    Normal,
}

/// Calculate the number of output frames given input count and frame skip.
///
/// `output_count = ceil(N / (frame_skip + 1))`.
pub fn calculate_required_frames(n_frames: usize, frame_skip: u32) -> usize {
    let skip = frame_skip as usize + 1;
    (n_frames / skip) + (n_frames % skip).min(1)
}

fn selected_output_frame_count(n_frames: usize, max_slots: usize, opts: &GenerateOptions) -> usize {
    requested_output_frame_count(n_frames, opts).min(max_slots)
}

fn requested_output_frame_count(n_frames: usize, opts: &GenerateOptions) -> usize {
    let available_frames = calculate_required_frames(n_frames, opts.frame_skip);
    if opts.trim_tail_for_exact_output_count {
        (opts.output_frames as usize).min(available_frames)
    } else {
        available_frames
    }
}

fn minimum_frame_skip_for_slots(n_frames: usize, max_slots: usize) -> u32 {
    if max_slots == 0 {
        return n_frames.saturating_sub(1) as u32;
    }
    for skip in 0..n_frames {
        if calculate_required_frames(n_frames, skip as u32) <= max_slots {
            return skip as u32;
        }
    }
    n_frames.saturating_sub(1) as u32
}

fn selected_input_prefix_len(
    n_frames: usize,
    total_output_frames: usize,
    opts: &GenerateOptions,
) -> usize {
    let step = opts.frame_skip as usize + 1;
    if opts.trim_tail_for_exact_output_count {
        return total_output_frames.saturating_mul(step).min(n_frames);
    }

    if total_output_frames == 0 {
        return 0;
    }
    let last_frame_idx = (total_output_frames - 1).saturating_mul(step);
    (last_frame_idx + 1).min(n_frames)
}

fn flow_progress_report_stride(total_batches: usize) -> usize {
    total_batches.div_ceil(64).max(1)
}

fn should_report_flow_progress(
    done: usize,
    total_batches: usize,
    report_stride: usize,
    highest_reported: &AtomicUsize,
) -> bool {
    if !(done.is_multiple_of(report_stride) || done == total_batches) {
        return false;
    }

    let mut current = highest_reported.load(Ordering::Relaxed);
    while done > current {
        match highest_reported.compare_exchange_weak(
            current,
            done,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return true,
            Err(actual) => current = actual,
        }
    }
    false
}

/// Run the full pipeline: frames → color atlas + motion atlas.
// allow(too_many_lines): pipeline orchestration is a single sequential workflow;
//   splitting would scatter the stage ordering across many functions
#[allow(clippy::too_many_lines)]
pub fn run_pipeline(
    frames: &dyn crate::io::FrameSource,
    opts: &GenerateOptions,
    progress: &(dyn Fn(Progress) + Sync),
    cancel: &(dyn Fn() -> bool + Sync),
) -> Result<EncodeResult, PipelineError> {
    // 1. Validate: need >= 2 frames
    if frames.len() < 2 {
        return Err(PipelineError::TooFewFrames(frames.len()));
    }

    let max_slots = (opts.atlas_dims.0 * opts.atlas_dims.1) as usize;
    let requested_frames = requested_output_frame_count(frames.len(), opts);
    if requested_frames > max_slots {
        return Err(PipelineError::AtlasOverflow {
            count: requested_frames,
            min_skip: minimum_frame_skip_for_slots(frames.len(), max_slots),
        });
    }

    let (source_width, source_height) = frames.dimensions();

    // 2. Premultiply alpha on each frame (parallelized)
    progress(Progress::Stage {
        name: "Premultiplying frames".to_string(),
        fraction: 0.0,
    });
    if cancel() {
        return Err(PipelineError::Cancelled);
    }

    let materialized: Vec<std::sync::Arc<ImageRgba8>> = (0..frames.len())
        .map(|i| frames.get(i))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| PipelineError::Other(format!("frame source: {e}")))?;

    let premul_counter = AtomicUsize::new(0);
    let total_frames_count = materialized.len();
    let premul_frames: Vec<ImageRgba8> = materialized
        .par_iter()
        .map(|f| {
            let img = premultiply_alpha(f.as_ref());
            let done = premul_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if done.is_multiple_of(8) || done == total_frames_count {
                progress(Progress::Stage {
                    name: "Premultiplying frames".to_string(),
                    fraction: done as f32 / total_frames_count as f32 * 0.1,
                });
            }
            img
        })
        .collect();

    // 3. Convert to grayscale (BT.601 via `OpenCV`-compatible fixed-point) — parallelized
    let mut gray_frames: Vec<ImageF32> = premul_frames.par_iter().map(rgba_to_gray_f32).collect();

    // 4. Compute atlas dimensions and frame layout
    let (atlas_cols, atlas_rows) = opts.atlas_dims;
    let frame_width = opts.tile_pixel_width;
    let extrude = opts.extrude.min((frame_width - 1) / 2);
    let valid_frame_width = frame_width - (extrude * 2);
    let valid_frame_height = predict_resize_height(source_height, source_width, valid_frame_width);
    let frame_height = valid_frame_height + (extrude * 2);

    let atlas_pixel_height = atlas_rows * frame_height;

    // 5. Compute output frame count
    let total_output_frames = selected_output_frame_count(gray_frames.len(), max_slots, opts);

    // Determine how many input frames to actually use for the pipeline.
    // The color atlas uses frames at indices 0, step, 2*step, ..., (total_output_frames-1)*step.
    // The motion atlas needs batches spanning those frames.
    // Batch loop terminates naturally when frame_idx >= len(frames)
    let frames_needed = selected_input_prefix_len(gray_frames.len(), total_output_frames, opts);

    // 6. Build color atlas. The atlas can be stored straight or premultiplied
    // depending on opts; flow analysis above already used the premultiplied
    // copy so transparent pixels contribute zero gradient regardless.
    let color_source: Vec<&ImageRgba8> = if opts.premultiplied_alpha {
        premul_frames[..frames_needed].iter().collect()
    } else {
        materialized[..frames_needed]
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect()
    };
    let color_atlas = build_color_atlas(
        &color_source,
        atlas_cols,
        atlas_rows,
        frame_width,
        frame_height,
        extrude,
        opts.frame_skip,
        opts.resize_algorithm,
    );

    // 6a. Lift low-contrast material above Farneback's fixed regularization
    // gate. One gain for the whole sequence — a per-frame gain would make the
    // frames incomparable and corrupt the flow. Resolves to 1.0 (a no-op) for
    // material that is already above the gate. See `flow::contrast`.
    //
    // Measured over the prefix the flow stage actually reads, not the full
    // decode: trailing frames that no batch touches would otherwise skew the
    // gain. Applied to the whole vector regardless — the tail is unused, and
    // scaling it keeps every frame on one common scale.
    let analysis_gain =
        crate::flow::contrast::analysis_gain(&gray_frames[..frames_needed], &opts.farneback);
    crate::flow::contrast::apply_gain(&mut gray_frames, analysis_gain);
    let gray_frames = gray_frames;

    // 7. Build motion vectors (only for the frames used in the atlas).
    // Farneback dominates the wall clock. Progress reports are throttled from
    // batch completion counters so Rayon still sees one full parallel batch.
    // Stage owns 10% → 85%.
    progress(Progress::Stage {
        name: "Computing flow".to_string(),
        fraction: 0.1,
    });
    if cancel() {
        return Err(PipelineError::Cancelled);
    }

    let plan = plan_batches(frames_needed, opts);
    let total_units = plan.batches.len() + 1; // batches + tail
    let completed_batches = AtomicUsize::new(0);
    let highest_reported = AtomicUsize::new(0);
    let report_stride = flow_progress_report_stride(plan.batches.len());
    let flow_progress = || {
        let done = completed_batches.fetch_add(1, Ordering::Relaxed) + 1;
        if should_report_flow_progress(done, plan.batches.len(), report_stride, &highest_reported) {
            progress(Progress::Stage {
                name: "Computing flow".to_string(),
                fraction: 0.75f32.mul_add(done as f32 / total_units as f32, 0.1),
            });
        }
    };
    let mut flows: Vec<Flow> = process_batch_slice_with_progress(
        &plan.batches,
        &gray_frames[..frames_needed],
        opts,
        &|| !cancel(),
        &flow_progress,
    );
    if cancel() {
        return Err(PipelineError::Cancelled);
    }
    flows.push(build_tail_flow_from_plan(
        &plan,
        &gray_frames[..frames_needed],
        opts,
    ));
    progress(Progress::Stage {
        name: "Computing flow".to_string(),
        fraction: 0.85,
    });
    if cancel() {
        return Err(PipelineError::Cancelled);
    }

    // Truncate to max_slots if we got more flows than tiles
    flows.truncate(max_slots);

    // 8. Process flows: clip, temporal smooth, then compute max_strength.
    let flow_count = flows.len();

    // 8a. Per-flow clip. The `1.0` is in NORMALIZED units (post `normalize_flow`
    // in process_batch_slice / build_tail_flow: dx/=width, dy/=height), not
    // pixels — i.e. cap displacement at 100% of the image dimension per output
    // pair. Sanity bound for flow blow-ups, not a quality knob; for typical
    // motion (a few px on ~hundreds-of-px frames) this is a no-op.
    for (i, flow) in flows.iter_mut().enumerate() {
        clip_flow_linf(flow, 1.0);
        progress(Progress::Stage {
            name: "Post-processing flow".to_string(),
            fraction: 0.05f32.mul_add((i + 1) as f32 / flow_count as f32, 0.85),
        });
        if cancel() {
            return Err(PipelineError::Cancelled);
        }
    }

    // 8b. Temporal smoothing across the flow sequence.
    if opts.temporal_smoothing > 0.0 {
        crate::pipeline::temporal::temporal_smooth(
            &mut flows,
            opts.temporal_smoothing,
            opts.is_loop,
        );
        progress(Progress::Stage {
            name: "Smoothing motion vectors".to_string(),
            fraction: 0.91,
        });
        if cancel() {
            return Err(PipelineError::Cancelled);
        }
    }

    // 8c. Recompute max_strength from the (possibly smoothed) flows.
    let mut max_strength: f32 = 0.0;
    for flow in &flows {
        update_max_strength(&mut max_strength, flow, opts.motion_vector_encoding);
    }

    // 9. Build motion atlas as float (resize flows into grid, then encode)
    progress(Progress::Stage {
        name: "Building atlases".to_string(),
        fraction: 0.92,
    });
    if cancel() {
        return Err(PipelineError::Cancelled);
    }
    let motion_vector_width = if opts.halve_motion_vector {
        // Floor the per-tile width to 1 px so a tiny tile_pixel_width can't
        // produce a zero-width (empty) motion atlas.
        (opts.tile_pixel_width / 2).max(1) * atlas_cols
    } else {
        opts.tile_pixel_width * atlas_cols
    };

    let motion_atlas = build_motion_atlas(
        &flows,
        atlas_cols,
        atlas_rows,
        frame_width,
        frame_height,
        extrude,
        motion_vector_width,
        max_strength,
        opts.motion_vector_encoding,
        opts.resize_algorithm,
    );

    // 10. Apply pack mode
    progress(Progress::Stage {
        name: "Encoding".to_string(),
        fraction: 0.97,
    });
    if cancel() {
        return Err(PipelineError::Cancelled);
    }
    let (packed_motion, pack_mode) = if opts.stagger_pack {
        (
            stagger_pack(
                &motion_atlas,
                atlas_cols,
                atlas_rows,
                opts.motion_vector_encoding,
            ),
            PackMode::Staggered,
        )
    } else {
        (flat_pack(&motion_atlas), PackMode::Normal)
    };

    progress(Progress::Done);

    Ok(EncodeResult {
        color_atlas,
        motion_atlas: packed_motion,
        strength: f64::from(max_strength),
        total_frames: total_output_frames as u32,
        atlas_width: opts.tile_pixel_width * atlas_cols,
        atlas_height: atlas_pixel_height,
        columns: atlas_cols,
        rows: atlas_rows,
        pack_mode,
        is_loop: opts.is_loop,
        premultiplied_alpha: opts.premultiplied_alpha,
        flows,
    })
}

/// Convert premultiplied RGBA8 to grayscale f32 with alpha-aware range split.
///
/// Transparent pixels → 0; opaque luma is remapped into [64, 255]. This
/// prevents opaque black/gray (e.g. smoke) from aliasing against transparent
/// regions in Farneback's polynomial expansion.
/// Base luma uses BT.601 fixed-point on the premultiplied RGB.
fn rgba_to_gray_f32(img: &ImageRgba8) -> ImageF32 {
    let mut result = ImageF32::zeros(img.width, img.height);
    for (i, chunk) in img.data.chunks_exact(4).enumerate() {
        let r = u32::from(chunk[0]);
        let g = u32::from(chunk[1]);
        let b = u32::from(chunk[2]);
        let a = u32::from(chunk[3]);
        // lum_premul = α · lum_orig, in [0, 255]
        let lum_premul = (r * 4899 + g * 9617 + b * 1868 + 8192) >> 14;
        // gray = α·64 + α·lum_orig·191/255 = α·64 + lum_premul·191/255
        // Range: transparent → 0; opaque → [64, 255] regardless of smoke color.
        let opaque_floor = a * 64 / 255;
        let lum_scaled = lum_premul * 191 / 255;
        result.data[i] = (opaque_floor + lum_scaled) as f32;
    }
    result
}

/// Predict the output height after Nyquist-safe resize to `new_width`.
///
/// For downscales: iteratively halves until close to target, then a single
/// final resize to exact dimensions. Avoids aliasing on 2–8× reductions
/// typical of flipbook content.
pub fn predict_resize_height(height: u32, width: u32, new_width: u32) -> u32 {
    if width == new_width {
        return height;
    }

    if width < new_width {
        // Upscale: single step. u64 intermediate avoids u32 overflow on large
        // height * new_width before the divide.
        return (u64::from(height) * u64::from(new_width) / u64::from(width)) as u32;
    }

    // Downscale: iterative halving
    let mut cur_h = height;
    let mut cur_w = width;

    while cur_w > new_width {
        let half_w = cur_w.div_ceil(2);
        let half_h = cur_h.div_ceil(2);

        if half_w <= new_width {
            cur_h = (f64::from(cur_h) * (f64::from(new_width) / f64::from(cur_w))).ceil() as u32;
            cur_w = new_width;
        } else {
            cur_h = half_h;
            cur_w = half_w;
        }
    }

    cur_h
}

/// Build the color atlas: resize frames and blit into grid.
/// Resize phase is parallelized (independent per tile); blits are serial (cheap memcpy).
// allow(too_many_arguments): atlas construction requires all parameters directly;
//   wrapping in struct would obscure the API for this internal function
#[allow(clippy::too_many_arguments)]
fn build_color_atlas(
    frames: &[&ImageRgba8],
    atlas_cols: u32,
    atlas_rows: u32,
    frame_width: u32,
    frame_height: u32,
    extrude: u32,
    frame_skip: u32,
    interp: crate::pipeline::Interpolation,
) -> ImageRgba8 {
    let atlas_w = atlas_cols * frame_width;
    let atlas_h = atlas_rows * frame_height;
    let mut atlas = ImageRgba8::zeros(atlas_w, atlas_h);

    let valid_width = frame_width - (extrude * 2);
    let step = frame_skip as usize + 1;

    // Collect indices for all tiles
    let mut tile_indices: Vec<usize> = vec![0];
    let mut i = step;
    while i < frames.len() {
        tile_indices.push(i);
        i += step;
    }

    // Parallelize the resize phase
    let resized_tiles: Vec<ImageRgba8> = tile_indices
        .par_iter()
        .map(|&frame_idx| resize_nyquist(frames[frame_idx], valid_width, interp))
        .collect();

    // Serial blit phase (cheap memcpy into atlas)
    for (atlas_idx, resized) in resized_tiles.iter().enumerate() {
        let tx = (atlas_idx as u32 % atlas_cols) * frame_width;
        let ty = (atlas_idx as u32 / atlas_cols) * frame_height;
        blit_extrude(&mut atlas, resized, tx, ty, extrude);
    }

    atlas
}

/// Build the motion atlas: resize normalized flows into grid, optionally halve, then encode.
/// Resize phase is parallelized (independent per tile); blits are serial.
// allow(too_many_arguments): all parameters needed for atlas geometry + encoding config
#[allow(clippy::too_many_arguments)]
fn build_motion_atlas(
    flows: &[Flow],
    atlas_cols: u32,
    atlas_rows: u32,
    frame_width: u32,
    frame_height: u32,
    extrude: u32,
    motion_vector_width: u32,
    max_strength: f32,
    encoding: MotionVectorEncoding,
    interp: crate::pipeline::Interpolation,
) -> ImageRgba8 {
    let atlas_w = atlas_cols * frame_width;
    let atlas_h = atlas_rows * frame_height;
    let valid_width = frame_width - (extrude * 2);

    // Build float atlas: resize each flow to tile dimensions and blit
    let valid_height = predict_resize_height(
        flows.first().map_or(1, |f| f.height),
        flows.first().map_or(1, |f| f.width),
        valid_width,
    );

    // Parallelize the resize phase (borrows flows; only clones if no resize needed)
    let resized_flows: Vec<Flow> = flows
        .par_iter()
        .map(|flow| resize_flow(flow, valid_width, valid_height, interp))
        .collect();

    // Serial blit phase
    let mut flow_atlas = Flow::zeros(atlas_w, atlas_h);
    for (idx, resized) in resized_flows.iter().enumerate() {
        let tx = (idx as u32 % atlas_cols) * frame_width;
        let ty = (idx as u32 / atlas_cols) * frame_height;
        blit_flow(
            &mut flow_atlas,
            resized,
            tx + extrude,
            ty + extrude,
            extrude,
        );
    }

    // If halving needed, resize the float atlas
    let final_atlas = if motion_vector_width == atlas_w {
        flow_atlas
    } else {
        resize_flow_atlas(&flow_atlas, motion_vector_width, atlas_rows, interp)
    };

    match encoding {
        MotionVectorEncoding::R8G8Remap01 => encode_r8g8_remap(&final_atlas, max_strength),
        MotionVectorEncoding::SidefxLabsR8G8 => encode_sidefx_labs(&final_atlas, max_strength),
    }
}

/// Resize a Flow field using Nyquist-safe iterative halving.
/// Final resize step dispatches on the specified interpolation method.
fn resize_flow(
    flow: &Flow,
    new_width: u32,
    new_height: u32,
    interp: crate::pipeline::Interpolation,
) -> Flow {
    if flow.width == new_width && flow.height == new_height {
        return flow.clone();
    }

    // Upscale or width-matches-height-differs: single-shot resize. The
    // Nyquist halving loop only applies when shrinking width, so anything
    // else routes straight through dispatch.
    if flow.width <= new_width {
        return resize_flow_dispatch(
            &flow.data,
            flow.width,
            flow.height,
            new_width,
            new_height,
            interp,
        );
    }

    // Downscale: iterative halving to respect Nyquist. First iteration reads
    // from `flow` directly; subsequent iterations chain through `owned`.
    let mut cur_w = flow.width;
    let mut cur_h = flow.height;
    let mut owned: Option<Flow> = None;

    while cur_w > new_width {
        let half_w = cur_w.div_ceil(2);
        let half_h = cur_h.div_ceil(2);
        let source: &Flow = owned.as_ref().unwrap_or(flow);

        if half_w <= new_width {
            let final_h =
                (f64::from(cur_h) * (f64::from(new_width) / f64::from(cur_w))).ceil() as u32;
            return resize_flow_dispatch(&source.data, cur_w, cur_h, new_width, final_h, interp);
        }
        owned = Some(resize_flow_dispatch(
            &source.data,
            cur_w,
            cur_h,
            half_w,
            half_h,
            interp,
        ));
        cur_w = half_w;
        cur_h = half_h;
    }

    owned.unwrap_or_else(|| flow.clone())
}

/// Dispatch flow resize to the appropriate algorithm.
fn resize_flow_dispatch(
    data: &[[f32; 2]],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
    interp: crate::pipeline::Interpolation,
) -> Flow {
    match interp {
        crate::pipeline::Interpolation::Nearest => {
            resize_flow_nearest_raw(data, src_w, src_h, dst_w, dst_h)
        }
        crate::pipeline::Interpolation::Linear => {
            resize_flow_bilinear_raw(data, src_w, src_h, dst_w, dst_h)
        }
        // Lanczos for flow data falls back to bicubic (Catmull-Rom); flow fields
        // are smooth enough that Lanczos lobes provide no benefit over bicubic.
        crate::pipeline::Interpolation::Cubic | crate::pipeline::Interpolation::Lanczos => {
            resize_flow_bicubic_raw(data, src_w, src_h, dst_w, dst_h)
        }
    }
}

/// Nearest-neighbor resize of flow data (2-channel f32).
fn resize_flow_nearest_raw(
    data: &[[f32; 2]],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Flow {
    let mut out = Flow::zeros(dst_w, dst_h);
    let sw = src_w as f32;
    let sh = src_h as f32;
    let dw = dst_w as f32;
    let dh = dst_h as f32;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = ((dx as f32 + 0.5) * sw / dw).floor() as u32;
            let sy = ((dy as f32 + 0.5) * sh / dh).floor() as u32;
            let sx = sx.min(src_w - 1);
            let sy = sy.min(src_h - 1);
            let src_idx = (sy as usize) * (src_w as usize) + (sx as usize);
            let dst_idx = (dy as usize) * (dst_w as usize) + (dx as usize);
            out.data[dst_idx] = data[src_idx];
        }
    }

    out
}

/// Bicubic (Catmull-Rom) resize of flow data (2-channel f32).
// allow(cast_possible_wrap): image dims always < i32::MAX (< 2GB pixel arrays)
#[allow(clippy::cast_possible_wrap)]
fn resize_flow_bicubic_raw(
    data: &[[f32; 2]],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Flow {
    let mut out = Flow::zeros(dst_w, dst_h);
    let sw = src_w as f32;
    let sh = src_h as f32;
    let dw = dst_w as f32;
    let dh = dst_h as f32;

    let catmull_rom = |t: f32| -> f32 {
        let t_abs = t.abs();
        if t_abs <= 1.0 {
            (1.5f32.mul_add(t_abs, -2.5) * t_abs).mul_add(t_abs, 1.0)
        } else if t_abs <= 2.0 {
            (-0.5f32)
                .mul_add(t_abs, 2.5)
                .mul_add(t_abs, -4.0)
                .mul_add(t_abs, 2.0)
        } else {
            0.0
        }
    };

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx as f32 + 0.5) * sw / dw - 0.5;
            let sy = (dy as f32 + 0.5) * sh / dh - 0.5;

            let ix = sx.floor() as i32;
            let iy = sy.floor() as i32;
            let fx = sx - ix as f32;
            let fy = sy - iy as f32;

            let mut sum = [0.0f32; 2];
            let mut weight_sum = 0.0f32;

            for ky in -1i32..=2 {
                let wy = catmull_rom(fy - ky as f32);
                for kx in -1i32..=2 {
                    let wx = catmull_rom(fx - kx as f32);
                    let w = wx * wy;
                    let cx = (ix + kx).clamp(0, src_w as i32 - 1) as usize;
                    let cy = (iy + ky).clamp(0, src_h as i32 - 1) as usize;
                    let val = data[cy * src_w as usize + cx];
                    sum[0] += val[0] * w;
                    sum[1] += val[1] * w;
                    weight_sum += w;
                }
            }

            let dst_idx = (dy as usize) * (dst_w as usize) + (dx as usize);
            if weight_sum > 0.0 {
                out.data[dst_idx] = [sum[0] / weight_sum, sum[1] / weight_sum];
            }
        }
    }

    out
}

/// Bilinear resize of flow data (2-channel f32).
// allow(cast_possible_wrap): image dims always < i32::MAX (< 2GB pixel arrays)
#[allow(clippy::cast_possible_wrap)]
fn resize_flow_bilinear_raw(
    data: &[[f32; 2]],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Flow {
    let sw = src_w as f32;
    let sh = src_h as f32;
    let dw = dst_w as f32;
    let dh = dst_h as f32;

    let mut out = Flow::zeros(dst_w, dst_h);

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx as f32 + 0.5) * sw / dw - 0.5;
            let sy = (dy as f32 + 0.5) * sh / dh - 0.5;

            let x0 = sx.floor() as i32;
            let y0 = sy.floor() as i32;
            let x1 = x0 + 1;
            let y1 = y0 + 1;

            let fx = sx - x0 as f32;
            let fy = sy - y0 as f32;

            let fetch = |ix: i32, iy: i32| -> [f32; 2] {
                let cx = ix.clamp(0, src_w as i32 - 1) as usize;
                let cy = iy.clamp(0, src_h as i32 - 1) as usize;
                data[cy * src_w as usize + cx]
            };

            let f00 = fetch(x0, y0);
            let f10 = fetch(x1, y0);
            let f01 = fetch(x0, y1);
            let f11 = fetch(x1, y1);

            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;

            let idx = (dy as usize) * (dst_w as usize) + (dx as usize);
            out.data[idx][0] =
                f00[0].mul_add(w00, f10[0].mul_add(w10, f01[0].mul_add(w01, f11[0] * w11)));
            out.data[idx][1] =
                f00[1].mul_add(w00, f10[1].mul_add(w10, f01[1].mul_add(w01, f11[1] * w11)));
        }
    }

    out
}

/// Blit a flow tile into the flow atlas at the given position with edge extrusion.
fn blit_flow(atlas: &mut Flow, tile: &Flow, ox: u32, oy: u32, extrude: u32) {
    let tw = tile.width;
    let th = tile.height;
    let aw = atlas.width;

    // Copy tile body
    for row in 0..th {
        for col in 0..tw {
            let src_idx = (row as usize) * (tw as usize) + (col as usize);
            let dst_x = ox + col;
            let dst_y = oy + row;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tile.data[src_idx];
        }
    }

    if extrude == 0 {
        return;
    }

    // Top extrusion
    for ey in 0..extrude {
        for col in 0..tw {
            let src_idx = col as usize; // row 0
            let dst_x = ox + col;
            let dst_y = oy - extrude + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tile.data[src_idx];
        }
    }

    // Bottom extrusion
    for ey in 0..extrude {
        for col in 0..tw {
            let src_idx = ((th - 1) as usize) * (tw as usize) + (col as usize);
            let dst_x = ox + col;
            let dst_y = oy + th + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tile.data[src_idx];
        }
    }

    // Left extrusion
    for row in 0..th {
        let src_idx = (row as usize) * (tw as usize); // col 0
        for ex in 0..extrude {
            let dst_x = ox - extrude + ex;
            let dst_y = oy + row;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tile.data[src_idx];
        }
    }

    // Right extrusion
    for row in 0..th {
        let src_idx = (row as usize) * (tw as usize) + ((tw - 1) as usize);
        for ex in 0..extrude {
            let dst_x = ox + tw + ex;
            let dst_y = oy + row;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tile.data[src_idx];
        }
    }

    // Corner extrusions
    let tl = tile.data[0];
    for ey in 0..extrude {
        for ex in 0..extrude {
            let dst_x = ox - extrude + ex;
            let dst_y = oy - extrude + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tl;
        }
    }

    let tr = tile.data[(tw - 1) as usize];
    for ey in 0..extrude {
        for ex in 0..extrude {
            let dst_x = ox + tw + ex;
            let dst_y = oy - extrude + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = tr;
        }
    }

    let bl = tile.data[((th - 1) as usize) * (tw as usize)];
    for ey in 0..extrude {
        for ex in 0..extrude {
            let dst_x = ox - extrude + ex;
            let dst_y = oy + th + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = bl;
        }
    }

    let br = tile.data[((th - 1) as usize) * (tw as usize) + ((tw - 1) as usize)];
    for ey in 0..extrude {
        for ex in 0..extrude {
            let dst_x = ox + tw + ex;
            let dst_y = oy + th + ey;
            let dst_idx = (dst_y as usize) * (aw as usize) + (dst_x as usize);
            atlas.data[dst_idx] = br;
        }
    }
}

/// Resize the entire flow atlas (for `halve_motion_vector`).
/// Resize the motion atlas to `new_width`, keeping the height an exact multiple
/// of `atlas_rows`. The atlas is `atlas_rows` tiles tall (`height = atlas_rows *
/// tile_h`); scaling height by the raw aspect ratio can yield a value not
/// divisible by `atlas_rows`, which makes `stagger_pack`'s floor-division
/// silently drop the bottom pixel rows. Snapping per-tile keeps every tile
/// boundary aligned.
fn resize_flow_atlas(
    atlas: &Flow,
    new_width: u32,
    atlas_rows: u32,
    interp: crate::pipeline::Interpolation,
) -> Flow {
    let ratio = f64::from(new_width) / f64::from(atlas.width);
    let rows = atlas_rows.max(1);
    let tile_h = atlas.height / rows; // exact: height == atlas_rows * tile_h by construction
    let new_tile_h = (f64::from(tile_h) * ratio).ceil() as u32;
    let new_height = new_tile_h * rows;
    resize_flow_dispatch(
        &atlas.data,
        atlas.width,
        atlas.height,
        new_width,
        new_height,
        interp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_required_frames_exact_divisions() {
        // 100 frames, skip 0 → (100/1) + min(0,1) = 100
        assert_eq!(calculate_required_frames(100, 0), 100);
        // 100 frames, skip 1 → (100/2) + min(0,1) = 50
        assert_eq!(calculate_required_frames(100, 1), 50);
        // 100 frames, skip 3 → (100/4) + min(0,1) = 25
        assert_eq!(calculate_required_frames(100, 3), 25);
    }

    #[test]
    fn calculate_required_frames_remainder() {
        // 7 frames, skip 1 → (7/2) + min(1,1) = 3 + 1 = 4
        assert_eq!(calculate_required_frames(7, 1), 4);
        // 5 frames, skip 2 → (5/3) + min(2,1) = 1 + 1 = 2
        assert_eq!(calculate_required_frames(5, 2), 2);
    }

    #[test]
    fn untrimmed_pipeline_uses_current_required_prefix() {
        let opts = GenerateOptions {
            frame_skip: 5,
            trim_tail_for_exact_output_count: false,
            ..Default::default()
        };

        assert_eq!(selected_output_frame_count(100, 100, &opts), 17);
        assert_eq!(selected_input_prefix_len(100, 17, &opts), 97);
    }

    #[test]
    fn trimmed_pipeline_uses_requested_output_count_and_exact_prefix() {
        let opts = GenerateOptions {
            output_frames: 16,
            frame_skip: 5,
            trim_tail_for_exact_output_count: true,
            ..Default::default()
        };

        assert_eq!(selected_output_frame_count(100, 100, &opts), 16);
        assert_eq!(selected_input_prefix_len(100, 16, &opts), 96);
    }

    #[test]
    fn selected_prefix_never_exceeds_available_frames() {
        let opts = GenerateOptions {
            output_frames: 16,
            frame_skip: 5,
            trim_tail_for_exact_output_count: true,
            ..Default::default()
        };

        assert_eq!(selected_output_frame_count(10, 100, &opts), 2);
        assert_eq!(selected_input_prefix_len(10, 16, &opts), 10);
    }

    #[test]
    fn pipeline_rejects_when_requested_frames_exceed_atlas_slots() {
        let frames = (0..100)
            .map(|i| ImageRgba8 {
                width: 32,
                height: 32,
                data: [i as u8, 0, 0, 255].repeat(1024),
            })
            .collect();
        let source = crate::io::InMemoryFrames::new(frames).unwrap();
        let opts = GenerateOptions {
            frame_skip: 0,
            atlas_dims: (8, 7),
            tile_pixel_width: 1,
            ..Default::default()
        };
        let progress = |_p: Progress| {};
        let cancel = || false;

        let Err(err) = run_pipeline(&source, &opts, &progress, &cancel) else {
            panic!("expected atlas overflow");
        };

        assert!(matches!(
            err,
            PipelineError::AtlasOverflow {
                count: 100,
                min_skip: 1
            }
        ));
    }

    #[test]
    fn flow_progress_report_stride_keeps_small_sequences_granular() {
        assert_eq!(flow_progress_report_stride(0), 1);
        assert_eq!(flow_progress_report_stride(1), 1);
        assert_eq!(flow_progress_report_stride(64), 1);
    }

    #[test]
    fn flow_progress_report_stride_caps_large_sequence_reports() {
        assert_eq!(flow_progress_report_stride(65), 2);
        assert_eq!(flow_progress_report_stride(128), 2);
        assert_eq!(flow_progress_report_stride(129), 3);
    }

    #[test]
    fn flow_progress_report_suppresses_out_of_order_lower_counts() {
        let highest_reported = AtomicUsize::new(0);

        assert!(should_report_flow_progress(128, 128, 64, &highest_reported));
        assert!(!should_report_flow_progress(64, 128, 64, &highest_reported));
    }

    #[test]
    fn predict_resize_height_identity() {
        assert_eq!(predict_resize_height(400, 400, 400), 400);
    }

    #[test]
    fn predict_resize_height_downscale() {
        // 400×400 → 128 wide: halving steps: 400→200→128 (200<=128? no. 200/2=100<=128? yes)
        // Actually: 400→200 (half). 200>128, half=100<=128 → ceil(200*(128/200)) = ceil(128) = 128
        let h = predict_resize_height(400, 400, 128);
        assert_eq!(h, 128);
    }

    #[test]
    fn predict_resize_height_upscale() {
        // 100×50 → 200 wide: upscale = 100 * 200 / 50 = 400
        assert_eq!(predict_resize_height(100, 50, 200), 400);
    }

    #[test]
    // allow(float_cmp): integer arithmetic produces exact f32 results, direct comparison safe
    #[allow(clippy::float_cmp)]
    fn gray_conversion_known_values() {
        // Alpha-aware mapping: transparent → 0, opaque luma → [64, 255].
        // gray = α·64/255 + lum_premul·191/255

        // Pure opaque white (255,255,255,255): lum_premul=255 → 64 + 191 = 255
        let img = ImageRgba8 {
            width: 1,
            height: 1,
            data: vec![255, 255, 255, 255],
        };
        let gray = rgba_to_gray_f32(&img);
        assert_eq!(gray.data[0], 255.0);

        // Pure opaque red (255,0,0,255): lum_premul=76 → 64 + 76·191/255 = 64 + 56 = 120
        let img_r = ImageRgba8 {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        let gray_r = rgba_to_gray_f32(&img_r);
        assert_eq!(gray_r.data[0], 120.0);

        // Opaque black (0,0,0,255): lum_premul=0 → 64 + 0 = 64 (smoke-tracking floor)
        let img_k = ImageRgba8 {
            width: 1,
            height: 1,
            data: vec![0, 0, 0, 255],
        };
        let gray_k = rgba_to_gray_f32(&img_k);
        assert_eq!(gray_k.data[0], 64.0);

        // Fully transparent (0,0,0,0): 0
        let img_t = ImageRgba8 {
            width: 1,
            height: 1,
            data: vec![0, 0, 0, 0],
        };
        let gray_t = rgba_to_gray_f32(&img_t);
        assert_eq!(gray_t.data[0], 0.0);
    }
}
