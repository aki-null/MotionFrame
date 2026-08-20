//! Uniform analysis gain for low-contrast source material.
//!
//! Farneback's per-pixel solve divides by `det + FLOW_REGULARIZATION`, where
//! `det` is the smoothed structure-tensor determinant (see
//! `update::solve_flow_parallel`). `det` scales as the 4th power of the input
//! image's local contrast while the regularizer is a fixed absolute value, so
//! the two together form a hard contrast gate. Under the gate the solve yields
//! ~0 and — because it overwrites rather than accumulates — discards the
//! coarser pyramid level's estimate. Motion in that region is lost outright.
//!
//! Photographic and fire/explosion content sits comfortably above the gate.
//! Content whose opaque interior is smooth does not: a smoke flipbook whose
//! shading is deliberately low-frequency and whose core alpha is saturated.
//! Measured on such a sheet, 62% of covered pixels fall below the gate at the finest
//! pyramid level, in one large contiguous blob, and the motion vectors there
//! come out at roughly a tenth of their true magnitude.
//!
//! The fix is a uniform gain on the analysis gray. Gain carries no new
//! information — it only moves `det` relative to the fixed regularizer, which
//! is exactly the quantity that is miscalibrated. Because `det ∝ gain⁴`, the
//! gain needed to place a chosen percentile of the determinant distribution a
//! chosen margin above the regularizer is available in closed form, so this
//! measures the real gate rather than a proxy for it.
//!
//! Two properties this module exists to protect:
//!
//! - The gain MUST be identical for every frame in the sequence. Farneback
//!   compares polynomial expansions between two frames; a per-frame gain would
//!   make them incomparable and corrupt the flow rather than improve it. This
//!   one is a hard invariant.
//! - Content already above the gate should come out near 1 and be effectively
//!   untouched, since the solve is scale-invariant once the regularizer is
//!   negligible. This is a design goal the statistic approximates, not a
//!   guarantee: the measured high-contrast sheets land at 1.1 to 2.2, and small
//!   over-gain there is harmless precisely because of that scale invariance.
//!   Large over-gain is not, which is what `DET_PERCENTILE` is tuned against.

use crate::flow::poly::poly_expansion;
use crate::flow::pyramid::build_level_image;
use crate::flow::update::{build_gaussian_1d, FLOW_REGULARIZATION};
use crate::pipeline::{FarnebackParams, ImageF32};
use rayon::prelude::*;

/// Frames sampled to estimate the sequence's structure scale. Contrast is a
/// property of the material, not of an individual frame, so a handful spread
/// across the sequence is enough and keeps this well under 1% of pipeline time.
const SAMPLE_FRAMES: usize = 8;

/// Target number of determinant SAMPLES kept per frame; the spatial stride is
/// derived from it.
///
/// This bounds the size of the collected distribution, NOT the work: the
/// expansion and the three smoothing passes still run over every pixel, because
/// the determinant is only meaningful after the same window averaging the solve
/// applies. Per-frame cost therefore scales with source area.
const SAMPLES_PER_FRAME: usize = 65_536;

/// Scratch-memory budget for the concurrent part of the measurement, in bytes.
///
/// `frame_determinants` keeps several full-resolution f32 buffers live at once,
/// so measuring N frames in parallel costs N times that. Deriving the
/// concurrency from a byte budget instead of pinning a frame count keeps small
/// tiles fully parallel while stopping large ones (2048² and up) from reserving
/// more than the wasm32 address space allows.
const SCRATCH_BUDGET_BYTES: usize = 256 << 20;

/// Peak scratch bytes per pixel held by one in-flight `frame_determinants`:
/// the level image, three product buffers, three smoothed outputs, and one
/// transient row buffer inside the separable pass.
const SCRATCH_BYTES_PER_PX: usize = 4 * 8;

/// How many sampled frames to measure concurrently for a given frame size.
fn concurrent_frames(pixels: usize) -> usize {
    let per_frame = pixels.saturating_mul(SCRATCH_BYTES_PER_PX).max(1);
    (SCRATCH_BUDGET_BYTES / per_frame).clamp(1, SAMPLE_FRAMES)
}

/// Percentile of the determinant distribution the gain is sized against.
///
/// The median, which makes the criterion "boost when the typical covered pixel
/// is below the gate". Whatever percentile is chosen becomes a cliff at that
/// coverage fraction — material whose flat share exceeds it gets boosted, and
/// below it does not — so the value decides where that cliff is safe to put.
///
/// The lower quartile was tried first and put the cliff too low. A frame that
/// is 40% solid interior and 60% sharp texture, far above the gate everywhere
/// that matters, demanded the full 8× — and one of the measured explosion
/// sheets did exactly that. Moving to the median dropped that sheet
/// from 8.0 to 1.1 and slightly IMPROVED its interpolation error, while costing
/// the low-contrast smoke about 1.4 points (gain 5.7 → 3.7). Trading a little
/// headroom on the case that needs help for not over-boosting the case that
/// does not is the right side of that trade.
///
/// Taken over pixels with a NON-ZERO determinant only — see
/// [`analysis_gain`] for why.
const DET_PERCENTILE: f64 = 0.50;

/// How far above the regularizer `DET_PERCENTILE` is placed. At 100× the
/// regularizer costs under 1% attenuation in the solve, i.e. that percentile
/// is fully inside the scale-invariant regime.
const TARGET_MARGIN: f64 = 100.0;

/// Upper bound on the gain.
///
/// Past roughly 8× the measured quality stops improving (the determinant is
/// already far above the gate everywhere that has real structure) while the
/// downside keeps growing: in genuinely featureless regions an unbounded gain
/// would lift 8-bit quantization steps into trackable structure and turn noise
/// into motion vectors.
const MAX_GAIN: f32 = 8.0;

/// Gray level at or above which a pixel counts as covered.
///
/// `rgba_to_gray_f32` maps fully transparent pixels to exactly 0 and gives any
/// non-zero alpha a floor contribution, so this threshold separates the
/// premultiplied-transparent background — which is uniformly black and would
/// otherwise dominate the determinant distribution with zeros — from actual
/// content.
const COVERED_MIN: f32 = 2.0;

/// Compute the uniform gain to apply to every analysis gray frame.
///
/// Returns `1.0` when the material is already above the contrast gate, which
/// is the common case; see the module docs for the mechanism.
///
/// Only pixels with a non-zero determinant enter the percentile. A zero means
/// the neighborhood is flat or purely one-dimensional, and gain multiplies zero
/// structure by a constant — it provably cannot rescue those pixels, so they
/// carry no information about how much gain the sequence needs. Including them
/// let any material with a large solid region (a flat-shaded interior, a wide
/// uniform core) push the quartile onto an exact zero and demand maximum gain
/// no matter how sharp the rest of the frame was, which is backwards: that is
/// the case where gain only amplifies quantization noise.
pub fn analysis_gain(frames: &[ImageF32], params: &FarnebackParams) -> f32 {
    if frames.is_empty() {
        return 1.0;
    }

    // Spread the samples across the whole sequence. `step_by` would clamp to a
    // stride of 1 for anything shorter than 2 * SAMPLE_FRAMES and only ever
    // look at the opening frames — and contrast usually falls off toward the
    // tail, which is exactly the part that needs measuring.
    let count = SAMPLE_FRAMES.min(frames.len());
    let sampled: Vec<&ImageF32> = (0..count)
        .map(|i| &frames[i * frames.len() / count])
        .collect();

    // Bounded concurrency: every in-flight frame holds several full-resolution
    // scratch buffers at once (see `frame_determinants`), so fanning all of
    // `sampled` out at once would multiply peak memory by `SAMPLE_FRAMES`.
    let pixels = (frames[0].width as usize).saturating_mul(frames[0].height as usize);
    let mut dets: Vec<f32> = sampled
        .chunks(concurrent_frames(pixels))
        .flat_map(|chunk| {
            chunk
                .par_iter()
                .flat_map(|frame| frame_determinants(frame, params))
                .collect::<Vec<f32>>()
        })
        .collect();

    if dets.is_empty() {
        // Either nothing is covered, or nothing anywhere has structure. Gain
        // cannot manufacture either one.
        return 1.0;
    }

    dets.sort_unstable_by(f32::total_cmp);
    let idx = ((dets.len() - 1) as f64 * DET_PERCENTILE).round() as usize;
    let det = f64::from(dets[idx]);

    let target = TARGET_MARGIN * f64::from(FLOW_REGULARIZATION);
    if det >= target {
        return 1.0;
    }

    // det ∝ gain⁴ — invert for the gain that lands `det` on `target`.
    // `det` is strictly positive here: zeros are filtered out upstream.
    ((target / det).powf(0.25) as f32).clamp(1.0, MAX_GAIN)
}

/// Apply `gain` to every frame in place.
///
/// Separate from [`analysis_gain`] so the caller can log or override the value,
/// and so the "same gain for all frames" invariant is visible at one call site.
pub fn apply_gain(frames: &mut [ImageF32], gain: f32) {
    if (gain - 1.0).abs() < f32::EPSILON {
        return;
    }
    frames.par_iter_mut().for_each(|frame| {
        for v in &mut frame.data {
            *v *= gain;
        }
    });
}

/// Sample the structure-tensor determinant across one frame at the finest
/// pyramid level.
///
/// The finest level is the binding one. Downsampling concentrates contrast into
/// fewer pixels, so `det` grows steeply with pyramid depth and coarse levels
/// clear the gate by orders of magnitude even on material that fails at level 0.
// allow(many_single_char_names): math vars (a, b, c) match the OpenCV notation
// used throughout `update.rs`, and w/h/n are the standard image extents.
#[allow(clippy::many_single_char_names)]
fn frame_determinants(gray: &ImageF32, params: &FarnebackParams) -> Vec<f32> {
    // Same image `farneback` feeds to the expansion at k=0: pyr_scale⁰ = 1, so
    // this blurs without resizing and dimensions stay aligned with `gray`.
    let level = build_level_image(gray, 0, params.pyr_scale);
    let poly = poly_expansion(&level, params.poly_n, params.poly_sigma);

    let w = level.width as usize;
    let h = level.height as usize;
    let n = w * h;
    if n == 0 {
        return Vec::new();
    }

    // Mirrors `update::build_matrices_parallel` with flow = 0 and poly1 ==
    // poly2. Using one frame's expansion for both is the standard
    // approximation: consecutive flipbook frames share their local structure,
    // which is the only property this statistic reads.
    let mut c0 = vec![0.0f32; n];
    let mut c1 = vec![0.0f32; n];
    let mut c2 = vec![0.0f32; n];
    for i in 0..n {
        let [r4, r6, r5, _, _] = poly.data[i];
        let a = r4;
        let b = r6 * 0.5;
        let c = r5;
        c0[i] = a.mul_add(a, b * b);
        c1[i] = b.mul_add(a, b * c);
        c2[i] = b.mul_add(b, c * c);
    }
    // Largest buffer here at 20 B/px, and nothing below reads it. Release it
    // before allocating the three smoothing outputs to hold peak down.
    drop(poly);

    let kernel = if params.use_gaussian {
        build_gaussian_1d(params.winsize)
    } else {
        vec![1.0f32; params.winsize.max(1) as usize]
    };
    let scale = if params.use_gaussian {
        1.0
    } else {
        1.0 / (params.winsize.max(1) * params.winsize.max(1)) as f32
    };

    let g11 = smooth_separable(&c0, w, h, &kernel);
    let g12 = smooth_separable(&c1, w, h, &kernel);
    let g22 = smooth_separable(&c2, w, h, &kernel);

    // Flat cost regardless of resolution: stride so ~SAMPLES_PER_FRAME pixels
    // are visited. Border attenuation (`compute_border_weight`) is skipped —
    // it touches a 5 px rim, far too small to move a quartile.
    let stride = ((n as f64 / SAMPLES_PER_FRAME as f64).sqrt().ceil() as usize).max(1);

    let mut out = Vec::with_capacity(SAMPLES_PER_FRAME);
    for y in (0..h).step_by(stride) {
        for x in (0..w).step_by(stride) {
            let i = y * w + x;
            if gray.data[i] < COVERED_MIN {
                continue;
            }
            let a = g11[i] * scale;
            let b = g12[i] * scale;
            let c = g22[i] * scale;
            let det = a.mul_add(c, -(b * b));
            // Drop non-positive determinants instead of clamping them to zero.
            // A flat neighborhood gives exactly 0 and a near-rank-1 one gives a
            // small negative from f32 cancellation; both mean "no structure to
            // scale up", and keeping them would let solid regions dominate the
            // quartile. See `analysis_gain`.
            if det > 0.0 {
                out.push(det);
            }
        }
    }
    out
}

/// Separable convolution with `BORDER_REPLICATE`, matching the averaging in
/// `update_flow_with_workspace`.
fn smooth_separable(src: &[f32], w: usize, h: usize, kernel: &[f32]) -> Vec<f32> {
    // `saturating_sub` then `min` is the BORDER_REPLICATE clamp of the tap
    // offset `x + j - half`, expressed without signed casts: undershoot pins to
    // column 0, overshoot pins to the last column.
    let half = kernel.len() / 2;
    let last_x = w.saturating_sub(1);
    let last_y = h.saturating_sub(1);

    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut acc = 0.0f32;
            for (j, kv) in kernel.iter().enumerate() {
                let nx = (x + j).saturating_sub(half).min(last_x);
                acc = src[row + nx].mul_add(*kv, acc);
            }
            tmp[row + x] = acc;
        }
    }

    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (j, kv) in kernel.iter().enumerate() {
                let ny = (y + j).saturating_sub(half).min(last_y);
                acc = tmp[ny * w + x].mul_add(*kv, acc);
            }
            out[y * w + x] = acc;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a frame with 2-D sinusoidal texture of the given amplitude over an
    /// opaque mid-gray field.
    ///
    /// The pattern must vary along both axes. A 1-D plane wave has a rank-1
    /// structure tensor, so its determinant is ~0 at any amplitude (the
    /// aperture problem) and it cannot stand in for high-contrast content.
    fn textured(w: u32, h: u32, amplitude: f32) -> ImageF32 {
        let mut img = ImageF32::zeros(w, h);
        for y in 0..h {
            for x in 0..w {
                let sx = ((x as f32) * 0.7).sin();
                let sy = ((y as f32) * 0.9).sin();
                img.data[(y * w + x) as usize] = 128.0 + amplitude * sx * sy;
            }
        }
        img
    }

    #[test]
    fn high_contrast_content_is_left_alone() {
        let params = FarnebackParams::default();
        let frames = vec![textured(96, 96, 60.0), textured(96, 96, 60.0)];
        let gain = analysis_gain(&frames, &params);
        assert!(
            (gain - 1.0).abs() < 1e-6,
            "high-contrast content must not be rescaled, got {gain}"
        );
    }

    #[test]
    fn low_contrast_content_is_boosted() {
        let params = FarnebackParams::default();
        let frames = vec![textured(96, 96, 0.35), textured(96, 96, 0.35)];
        let gain = analysis_gain(&frames, &params);
        assert!(
            gain > 1.0,
            "low-contrast content must be boosted, got {gain}"
        );
        assert!(gain <= MAX_GAIN, "gain must stay clamped, got {gain}");
    }

    #[test]
    fn gain_is_monotonic_in_contrast() {
        let params = FarnebackParams::default();
        let weak = analysis_gain(&[textured(96, 96, 0.2)], &params);
        let strong = analysis_gain(&[textured(96, 96, 2.0)], &params);
        assert!(
            weak >= strong,
            "weaker contrast must not get less gain: {weak} vs {strong}"
        );
    }

    #[test]
    fn large_flat_interior_does_not_force_max_gain() {
        // A solid interior plus sharp texture is high-contrast content: the
        // textured part is far above the gate and gain cannot help the flat
        // part at all. Sizing the gain off a low percentile used to read the
        // flat share as "weak structure" and demand the full 8x, which only
        // amplifies quantization noise there.
        let params = FarnebackParams::pipeline_tuned();
        let mut img = ImageF32::zeros(128, 128);
        let split = 128 * 2 / 5; // 40% flat
        for y in 0..128u32 {
            for x in 0..128u32 {
                let v = if x < split {
                    128.0
                } else {
                    let sx = ((x as f32) * 0.7).sin();
                    let sy = ((y as f32) * 0.9).sin();
                    (60.0f32).mul_add(sx * sy, 128.0).round()
                };
                img.data[(y * 128 + x) as usize] = v;
            }
        }
        let gain = analysis_gain(&[img], &params);
        assert!(
            (gain - 1.0).abs() < 1e-6,
            "40% flat + sharp texture must not be boosted, got {gain}"
        );
    }

    #[test]
    fn samples_span_the_whole_sequence() {
        // A sequence shorter than 2 * SAMPLE_FRAMES must still be measured
        // across its full length. A stride-based sampler collapses to stride 1
        // here and never looks past frame 7.
        //
        // The leading frames are fully transparent so they contribute no
        // determinant samples at all (COVERED_MIN filters them), which leaves
        // the tail as the only thing the statistic can see. A prefix-only
        // sampler therefore measures nothing and returns 1.0; a spread sampler
        // finds the low-contrast tail and boosts.
        let params = FarnebackParams::pipeline_tuned();
        let mut frames: Vec<ImageF32> = (0..12).map(|_| ImageF32::zeros(96, 96)).collect();
        for f in frames.iter_mut().skip(8) {
            *f = textured(96, 96, 0.35);
        }
        let gain = analysis_gain(&frames, &params);
        assert!(
            gain > 1.0,
            "low-contrast tail of a 12-frame sequence must be sampled, got {gain}"
        );
    }

    #[test]
    fn fully_transparent_sequence_yields_unit_gain() {
        // Every pixel below COVERED_MIN: nothing to measure, nothing to boost.
        let params = FarnebackParams::default();
        let frames = vec![ImageF32::zeros(64, 64), ImageF32::zeros(64, 64)];
        let gain = analysis_gain(&frames, &params);
        assert!(
            (gain - 1.0).abs() < 1e-6,
            "empty sequence must yield unit gain, got {gain}"
        );
    }

    #[test]
    fn empty_sequence_yields_unit_gain() {
        let params = FarnebackParams::default();
        assert!((analysis_gain(&[], &params) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_gain_scales_every_frame_identically() {
        let mut frames = vec![textured(16, 16, 4.0), textured(16, 16, 4.0)];
        let before: Vec<f32> = frames[0].data.clone();
        apply_gain(&mut frames, 3.0);
        for (i, v) in frames[0].data.iter().enumerate() {
            assert!((v - before[i] * 3.0).abs() < 1e-3);
        }
        // Both frames must carry the same scaling, or Farneback's frame-to-frame
        // comparison is invalid.
        assert_eq!(frames[0].data, frames[1].data);
    }

    #[test]
    fn concurrency_scales_down_with_frame_size() {
        // Small tiles keep full parallelism; large ones collapse toward serial
        // so peak scratch stays inside the budget.
        assert_eq!(concurrent_frames(128 * 128), SAMPLE_FRAMES);
        assert_eq!(concurrent_frames(512 * 512), SAMPLE_FRAMES);
        assert_eq!(concurrent_frames(1024 * 1024), SAMPLE_FRAMES);
        // 2048²: 134 MB per frame, so two fit the 256 MB budget.
        assert_eq!(concurrent_frames(2048 * 2048), 2);
        // 4096²: one frame already exceeds the budget — floor at serial.
        assert_eq!(concurrent_frames(4096 * 4096), 1);
        assert_eq!(concurrent_frames(0), SAMPLE_FRAMES);
        for px in [1usize, 64, 4096, 1 << 24, usize::MAX] {
            let n = concurrent_frames(px);
            assert!((1..=SAMPLE_FRAMES).contains(&n), "px {px} gave {n}");
        }
    }

    #[test]
    fn apply_gain_of_one_is_a_no_op() {
        let mut frames = vec![textured(16, 16, 4.0)];
        let before = frames[0].data.clone();
        apply_gain(&mut frames, 1.0);
        assert_eq!(frames[0].data, before);
    }
}
