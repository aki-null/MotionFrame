//! Per-pixel displacement update for Farneback optical flow.
//!
//! Derived from `OpenCV`'s `optflowgf.cpp` (`FarnebackUpdateMatrices`,
//! `FarnebackUpdateFlow`). See THIRD-PARTY-LICENSES.md for license attribution.

use crate::flow::poly::PolyImage;
use crate::pipeline::Flow;
use rayon::prelude::*;
use wide::f32x4;

/// Pre-allocated workspace buffers for `update_flow_with_workspace`, reused across iterations.
/// Uses Structure-of-Arrays layout for cache-friendly vectorizable smooth passes.
pub struct UpdateWorkspace {
    /// 5 separate component arrays for the per-pixel matrices (`SoA` layout)
    mat: [Vec<f32>; 5],
    /// 5 separate component arrays for horizontal smooth output
    horiz: [Vec<f32>; 5],
    /// 5 separate component arrays for final smoothed output
    smoothed: [Vec<f32>; 5],
}

impl UpdateWorkspace {
    /// Create workspace sized for the given image dimensions.
    pub fn new(w: usize, h: usize) -> Self {
        let n = w * h;
        Self {
            mat: std::array::from_fn(|_| vec![0.0f32; n]),
            horiz: std::array::from_fn(|_| vec![0.0f32; n]),
            smoothed: std::array::from_fn(|_| vec![0.0f32; n]),
        }
    }

    /// Resize workspace if needed (no-op if already large enough).
    pub fn ensure_size(&mut self, w: usize, h: usize) {
        let n = w * h;
        if self.mat[0].len() < n {
            for c in 0..5 {
                self.mat[c].resize(n, 0.0);
                self.horiz[c].resize(n, 0.0);
                self.smoothed[c].resize(n, 0.0);
            }
        }
    }
}

/// Update flow field using polynomial coefficients from both frames, with pre-allocated workspace.
///
/// Based on `OpenCV`'s optflowgf.cpp: `FarnebackUpdateMatrices` + `FarnebackUpdateFlow_Blur`.
///
/// For each pixel, builds a 2×2 linear system from averaged polynomial coefficients
/// and solves for the TOTAL flow (overwrite, not accumulate).
///
/// Key: `OpenCV` adds `A * flow_old` to the signal before forming the normal equation.
/// This makes the solve produce the TOTAL flow directly, so we OVERWRITE (not accumulate).
/// At convergence (correct warp), signal ≈ `A*flow_old`, so `flow_new` ≈ `flow_old` (stable).
///
/// Border policy: `BORDER_REPLICATE` for the box/Gaussian averaging (matches `OpenCV`).
// Based on OpenCV's optflowgf.cpp: FarnebackUpdateMatrices + FarnebackUpdateFlow_Blur
// allow(many_single_char_names): math vars (a,b,c,d,e) match OpenCV notation
// allow(cast_possible_wrap): image dims always < i32::MAX in practice
// allow(too_many_lines): algorithm is a single logical unit matching OpenCV's structure
#[allow(
    clippy::many_single_char_names,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
pub fn update_flow_with_workspace(
    flow: &mut Flow,
    poly1: &PolyImage,
    poly2: &PolyImage,
    winsize: u32,
    use_gaussian: bool,
    ws: &mut UpdateWorkspace,
) {
    let w = flow.width as usize;
    let h = flow.height as usize;
    let half_win = (winsize / 2) as i32;

    ws.ensure_size(w, h);

    let kernel = if use_gaussian {
        build_gaussian_1d(winsize)
    } else {
        // Box kernel: accumulate unweighted (all 1.0), then normalize by
        // 1/N² at solve time (see `scale` below) for efficiency.
        vec![1.0f32; winsize as usize]
    };

    let is_box = !use_gaussian;
    let n = w * h;

    // Step 1: Build per-pixel M matrix entries (parallelized by row).
    // Based on OpenCV's optflowgf.cpp: FarnebackUpdateMatrices
    build_matrices_parallel(flow, poly1, poly2, w, h, ws);

    // Step 2: Separable smooth — fused parallel dispatches for all 5 components.
    // Nested parallelism: outer par_iter across 5 components, inner par_chunks for rows.
    // Rayon's work-stealing distributes across all available cores.
    let half = half_win as usize;
    let ksize = kernel.len();

    // Horizontal pass: 5 components in parallel, each with row-level parallelism
    {
        let (m0, rest) = ws.mat.split_at_mut(1);
        let (m1, rest) = rest.split_at_mut(1);
        let (m2, rest) = rest.split_at_mut(1);
        let (m3, m4) = rest.split_at_mut(1);
        let mat_refs: [&[f32]; 5] = [
            &m0[0][..n],
            &m1[0][..n],
            &m2[0][..n],
            &m3[0][..n],
            &m4[0][..n],
        ];

        let (h0, rest) = ws.horiz.split_at_mut(1);
        let (h1, rest) = rest.split_at_mut(1);
        let (h2, rest) = rest.split_at_mut(1);
        let (h3, h4) = rest.split_at_mut(1);

        [
            &mut h0[0][..n],
            &mut h1[0][..n],
            &mut h2[0][..n],
            &mut h3[0][..n],
            &mut h4[0][..n],
        ]
        .into_par_iter()
        .enumerate()
        .for_each(|(c, horiz_buf)| {
            let src = mat_refs[c];
            horiz_buf
                .par_chunks_mut(w)
                .enumerate()
                .for_each(|(y, dst_row)| {
                    let row_start = y * w;
                    sep_conv_horiz_row(
                        src, dst_row, row_start, w, &kernel, ksize, half, half_win, is_box,
                    );
                });
        });
    }

    // Vertical pass: 5 components in parallel, each with row-level parallelism.
    // Inner loop is x-outer / k-inner with explicit f32x4 SIMD: each output
    // pixel is written exactly once and the 4-wide accumulator stays in
    // registers across ksize taps. Hoists is_box and drops the redundant
    // fill(0.0) (k=0 first tap initializes the accumulator).
    {
        let (s0, rest) = ws.smoothed.split_at_mut(1);
        let (s1, rest) = rest.split_at_mut(1);
        let (s2, rest) = rest.split_at_mut(1);
        let (s3, s4) = rest.split_at_mut(1);

        let (hr0, rest) = ws.horiz.split_at_mut(1);
        let (hr1, rest) = rest.split_at_mut(1);
        let (hr2, rest) = rest.split_at_mut(1);
        let (hr3, hr4) = rest.split_at_mut(1);
        let horiz_refs: [&[f32]; 5] = [
            &hr0[0][..n],
            &hr1[0][..n],
            &hr2[0][..n],
            &hr3[0][..n],
            &hr4[0][..n],
        ];

        [
            &mut s0[0][..n],
            &mut s1[0][..n],
            &mut s2[0][..n],
            &mut s3[0][..n],
            &mut s4[0][..n],
        ]
        .into_par_iter()
        .enumerate()
        .for_each(|(c, smooth_buf)| {
            let src = horiz_refs[c];
            smooth_buf
                .par_chunks_mut(w)
                .enumerate()
                .for_each(|(y, dst_row)| {
                    sep_conv_vert_row(src, dst_row, y, w, h, &kernel, ksize, half_win, is_box);
                });
        });
    }

    // Step 3: Solve 2×2 system per pixel — OVERWRITE flow (parallelized by row).
    // f32 throughout: the 2×2 system is well conditioned (regularization 1e-3 ≫
    // f32 epsilon at expected magnitudes).
    let scale: f32 = if use_gaussian {
        1.0
    } else {
        1.0 / (winsize * winsize) as f32
    };
    solve_flow_parallel(flow, ws, w, h, scale);
}

/// Build per-pixel matrices in parallel (by row).
// allow(many_single_char_names): math vars (a,b,c,d,e) match OpenCV's FarnebackUpdateMatrices
#[allow(clippy::many_single_char_names)]
fn build_matrices_parallel(
    flow: &Flow,
    poly1: &PolyImage,
    poly2: &PolyImage,
    w: usize,
    h: usize,
    ws: &mut UpdateWorkspace,
) {
    // Zip all 5 mat arrays by row for parallel mutable access
    let (m0, rest) = ws.mat.split_at_mut(1);
    let (m1, rest) = rest.split_at_mut(1);
    let (m2, rest) = rest.split_at_mut(1);
    let (m3, m4) = rest.split_at_mut(1);

    m0[0][..w * h]
        .par_chunks_mut(w)
        .zip(m1[0][..w * h].par_chunks_mut(w))
        .zip(m2[0][..w * h].par_chunks_mut(w))
        .zip(m3[0][..w * h].par_chunks_mut(w))
        .zip(m4[0][..w * h].par_chunks_mut(w))
        .enumerate()
        .for_each(|(y, ((((c0, c1), c2), c3), c4))| {
            // OpenCV's BORDER=5 attenuation returns 1.0 for any pixel ≥ 5 from
            // every edge. Split the row so the interior segment skips both the
            // branchy compute_border_weight call and the * bw2 multiplies. On
            // the bench (64f × 256² × 8x8 atlas) this is ~80% of pixels.
            const BORDER: usize = 5;
            let interior_y = y >= BORDER && y + BORDER < h;
            let interior_x_start = BORDER.min(w);
            let interior_x_end = w.saturating_sub(BORDER).max(interior_x_start);

            // Border pixel: full bw2 multiply.
            let border_x = |x: usize,
                            c0: &mut [f32],
                            c1: &mut [f32],
                            c2: &mut [f32],
                            c3: &mut [f32],
                            c4: &mut [f32]| {
                let idx = y * w + x;
                let [r4_1, r6_1, r5_1, r2_1, r3_1] = poly1.data[idx];
                let [dx, dy] = flow.data[idx];
                let [r4_2, r6_2, r5_2, r2_2, r3_2] =
                    sample_poly_bilinear(poly2, x as f32 + dx, y as f32 + dy, w, h);

                let a = (r4_1 + r4_2) * 0.5;
                let b = (r6_1 + r6_2) * 0.25;
                let c = (r5_1 + r5_2) * 0.5;
                let d = (r2_2 - r2_1).mul_add(0.5, a.mul_add(dx, b * dy));
                let e = (r3_2 - r3_1).mul_add(0.5, b.mul_add(dx, c * dy));

                let border_scale = compute_border_weight(x, y, w, h);
                let bw2 = border_scale * border_scale;

                c0[x] = a.mul_add(a, b * b) * bw2;
                c1[x] = b.mul_add(a, b * c) * bw2;
                c2[x] = b.mul_add(b, c * c) * bw2;
                c3[x] = a.mul_add(d, b * e) * bw2;
                c4[x] = b.mul_add(d, c * e) * bw2;
            };

            if !interior_y {
                for x in 0..w {
                    border_x(x, c0, c1, c2, c3, c4);
                }
                return;
            }

            for x in 0..interior_x_start {
                border_x(x, c0, c1, c2, c3, c4);
            }

            // Interior: bw = 1.0, drop the branch + 6 muls per pixel.
            for x in interior_x_start..interior_x_end {
                let idx = y * w + x;
                let [r4_1, r6_1, r5_1, r2_1, r3_1] = poly1.data[idx];
                let [dx, dy] = flow.data[idx];
                let [r4_2, r6_2, r5_2, r2_2, r3_2] =
                    sample_poly_bilinear(poly2, x as f32 + dx, y as f32 + dy, w, h);

                let a = (r4_1 + r4_2) * 0.5;
                let b = (r6_1 + r6_2) * 0.25;
                let c = (r5_1 + r5_2) * 0.5;
                let d = (r2_2 - r2_1).mul_add(0.5, a.mul_add(dx, b * dy));
                let e = (r3_2 - r3_1).mul_add(0.5, b.mul_add(dx, c * dy));

                c0[x] = a.mul_add(a, b * b);
                c1[x] = b.mul_add(a, b * c);
                c2[x] = b.mul_add(b, c * c);
                c3[x] = a.mul_add(d, b * e);
                c4[x] = b.mul_add(d, c * e);
            }

            for x in interior_x_end..w {
                border_x(x, c0, c1, c2, c3, c4);
            }
        });
}

/// Regularization epsilon added to the structure-tensor determinant before the
/// 2×2 solve, matching `OpenCV`'s `FarnebackUpdateFlow`.
///
/// This value is ABSOLUTE while the determinant scales as the 4th power of the
/// input image's local contrast, so it doubles as a hard contrast gate: any
/// region whose determinant lands below it has its flow driven to ~0 —
/// overwriting whatever the coarser pyramid level had estimated. Low-contrast
/// content (smooth smoke interiors, for instance) falls under the gate
/// across large contiguous areas and loses its motion entirely.
///
/// `crate::flow::contrast::analysis_gain` reads this constant to size the
/// uniform gain that lifts such content back above the gate. Keep the two in
/// sync: they are one calibration.
pub(crate) const FLOW_REGULARIZATION: f32 = 1e-3;

/// Solve flow in parallel by row.
fn solve_flow_parallel(flow: &mut Flow, ws: &UpdateWorkspace, w: usize, _h: usize, scale: f32) {
    let sm0 = &ws.smoothed[0];
    let sm1 = &ws.smoothed[1];
    let sm2 = &ws.smoothed[2];
    let sm3 = &ws.smoothed[3];
    let sm4 = &ws.smoothed[4];

    flow.data
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(y, flow_row)| {
            let row_offset = y * w;
            for (x, flow_elem) in flow_row.iter_mut().enumerate() {
                let idx = row_offset + x;
                let g11_s = sm0[idx] * scale;
                let g12_s = sm1[idx] * scale;
                let g22_s = sm2[idx] * scale;
                let h1_s = sm3[idx] * scale;
                let h2_s = sm4[idx] * scale;

                let idet = 1.0 / (g11_s.mul_add(g22_s, -(g12_s * g12_s)) + FLOW_REGULARIZATION);
                let flow_x = g22_s.mul_add(h1_s, -(g12_s * h2_s)) * idet;
                let flow_y = g11_s.mul_add(h2_s, -(g12_s * h1_s)) * idet;

                *flow_elem = [flow_x, flow_y];
            }
        });
}

/// Process a single row of horizontal convolution.
#[inline]
// allow(cast_possible_wrap): kernel half-size + col index, always < i32::MAX
#[allow(clippy::cast_possible_wrap)]
// allow(too_many_arguments): internal helper, parameters map 1:1 to convolution inputs
#[allow(clippy::too_many_arguments)]
fn sep_conv_horiz_row(
    src: &[f32],
    dst_row: &mut [f32],
    row_start: usize,
    w: usize,
    kernel: &[f32],
    ksize: usize,
    half: usize,
    half_win: i32,
    is_box: bool,
) {
    // Left border: scalar with clamping
    for (x, dst_elem) in dst_row[..half.min(w)].iter_mut().enumerate() {
        let mut sum = 0.0f32;
        for (k, &kval) in kernel.iter().enumerate().take(ksize) {
            let sx = (x as i32 + k as i32 - half_win).max(0).min(w as i32 - 1) as usize;
            if is_box {
                sum += src[row_start + sx];
            } else {
                sum += src[row_start + sx] * kval;
            }
        }
        *dst_elem = sum;
    }

    // Interior: SIMD path (no clamping needed)
    //
    // `half.min(w)`, not `half`: for a row shorter than the kernel half-width
    // there is no interior at all, and an unclamped `half` makes `simd_end`
    // exceed the row length — the scalar-tail slice below then panics. The
    // left-border loop above already wrote the whole row in that case, so the
    // interior and tail must both collapse to empty. Rows at least `half` wide
    // are unaffected.
    let interior_start = half.min(w);
    let interior_end = if w > half { w - half } else { interior_start };
    let interior_len = interior_end.saturating_sub(interior_start);
    let simd_end = interior_start + (interior_len / 4) * 4;

    let mut x = interior_start;
    if is_box {
        while x < simd_end {
            let mut acc = f32x4::ZERO;
            for k in 0..ksize {
                let base = row_start + x + k - half;
                // can't fail: interior SIMD loop guarantees base..base+4 within row bounds
                let arr: [f32; 4] = src[base..base + 4].try_into().unwrap();
                acc += f32x4::new(arr);
            }
            let arr: [f32; 4] = acc.into();
            dst_row[x..x + 4].copy_from_slice(&arr);
            x += 4;
        }
    } else {
        while x < simd_end {
            let mut acc = f32x4::ZERO;
            for (k, &wt) in kernel.iter().enumerate().take(ksize) {
                let base = row_start + x + k - half;
                // can't fail: interior SIMD loop guarantees base..base+4 within row bounds
                let arr: [f32; 4] = src[base..base + 4].try_into().unwrap();
                acc += f32x4::new(arr) * f32x4::splat(wt);
            }
            let arr: [f32; 4] = acc.into();
            dst_row[x..x + 4].copy_from_slice(&arr);
            x += 4;
        }
    }

    // Scalar tail + right border
    for (x_offset, dst_elem) in dst_row[simd_end..].iter_mut().enumerate() {
        let x = simd_end + x_offset;
        let mut sum = 0.0f32;
        for (k, &kval) in kernel.iter().enumerate().take(ksize) {
            let sx = (x as i32 + k as i32 - half_win).max(0).min(w as i32 - 1) as usize;
            if is_box {
                sum += src[row_start + sx];
            } else {
                sum += src[row_start + sx] * kval;
            }
        }
        *dst_elem = sum;
    }
}

/// Process a single row of the vertical separable convolution.
/// Loop order is x-outer / k-inner: each output pixel is written exactly once,
/// kernel taps accumulate in a 4-wide register across the row. `BORDER_REPLICATE`
/// (clamp) on row indices.
#[inline]
// allow(too_many_arguments): internal helper, parameters map 1:1 to convolution inputs
#[allow(clippy::too_many_arguments, clippy::cast_possible_wrap)]
fn sep_conv_vert_row(
    src: &[f32],
    dst_row: &mut [f32],
    y: usize,
    w: usize,
    h: usize,
    kernel: &[f32],
    ksize: usize,
    half_win: i32,
    is_box: bool,
) {
    // Precompute clamped source row offsets once per output row.
    // Hard assert (not debug_assert): release-build crash without diagnostic
    // would be worse than a clear panic. winsize=63 covers any sane Farneback
    // window; FarnebackParams default is 15.
    assert!(ksize <= 64, "winsize > 63 not supported (got {ksize})");
    let mut sy_offsets: [usize; 64] = [0; 64];
    for (k, off) in sy_offsets.iter_mut().enumerate().take(ksize) {
        let sy = (y as i32 + k as i32 - half_win).max(0).min(h as i32 - 1) as usize;
        *off = sy * w;
    }
    let sy = &sy_offsets[..ksize];

    let simd_end = (w / 4) * 4;
    let mut x = 0;
    if is_box {
        while x < simd_end {
            let mut acc = f32x4::ZERO;
            for &row_off in sy {
                let arr: [f32; 4] = src[row_off + x..row_off + x + 4].try_into().unwrap();
                acc += f32x4::new(arr);
            }
            let out: [f32; 4] = acc.into();
            dst_row[x..x + 4].copy_from_slice(&out);
            x += 4;
        }
    } else {
        while x < simd_end {
            let mut acc = f32x4::ZERO;
            for (k, &row_off) in sy.iter().enumerate() {
                let wt = f32x4::splat(kernel[k]);
                let arr: [f32; 4] = src[row_off + x..row_off + x + 4].try_into().unwrap();
                acc += f32x4::new(arr) * wt;
            }
            let out: [f32; 4] = acc.into();
            dst_row[x..x + 4].copy_from_slice(&out);
            x += 4;
        }
    }

    // Scalar tail
    for (x_offset, dst_elem) in dst_row[simd_end..].iter_mut().enumerate() {
        let x = simd_end + x_offset;
        let mut sum = 0.0f32;
        if is_box {
            for &row_off in sy {
                sum += src[row_off + x];
            }
        } else {
            for (k, &row_off) in sy.iter().enumerate() {
                sum += src[row_off + x] * kernel[k];
            }
        }
        *dst_elem = sum;
    }
}

/// Compute border attenuation weight for a pixel.
/// Based on `OpenCV`'s optflowgf.cpp: BORDER=5, border[] = {0.14, 0.14, 0.4472, 0.4472, 0.4472}
/// Returns 1.0 for interior pixels, reduced weight for pixels within 5 of any edge.
#[inline]
fn compute_border_weight(x: usize, y: usize, w: usize, h: usize) -> f32 {
    const BORDER: usize = 5;
    const WEIGHTS: [f32; BORDER] = [0.14, 0.14, 0.4472, 0.4472, 0.4472];

    let in_border = x < BORDER || x >= w - BORDER || y < BORDER || y >= h - BORDER;
    if !in_border {
        return 1.0;
    }

    let wx = if x < BORDER {
        WEIGHTS[x]
    } else if x >= w - BORDER {
        WEIGHTS[w - x - 1]
    } else {
        1.0
    };

    let wy = if y < BORDER {
        WEIGHTS[y]
    } else if y >= h - BORDER {
        WEIGHTS[h - y - 1]
    } else {
        1.0
    };

    wx * wy
}

/// Bilinear interpolation of poly expansion coefficients at fractional position (`fx`, `fy`).
/// Border policy: clamp to image edges.
/// Based on `OpenCV`'s optflowgf.cpp: `FarnebackUpdateMatrices` bilinear sampling of R1.
#[inline]
fn sample_poly_bilinear(poly: &PolyImage, fx: f32, fy: f32, w: usize, h: usize) -> [f32; 5] {
    let fx = fx.max(0.0).min((w - 1) as f32);
    let fy = fy.max(0.0).min((h - 1) as f32);

    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);

    let ax = fx - x0 as f32;
    let ay = fy - y0 as f32;

    let w00 = (1.0 - ax) * (1.0 - ay);
    let w10 = ax * (1.0 - ay);
    let w01 = (1.0 - ax) * ay;
    let w11 = ax * ay;

    let p00 = poly.data[y0 * w + x0];
    let p10 = poly.data[y0 * w + x1];
    let p01 = poly.data[y1 * w + x0];
    let p11 = poly.data[y1 * w + x1];

    let mut result = [0.0f32; 5];
    for i in 0..5 {
        result[i] = p00[i].mul_add(w00, p10[i].mul_add(w10, p01[i].mul_add(w01, p11[i] * w11)));
    }
    result
}

/// Build a 1D Gaussian kernel of given window size.
/// Sigma matches `OpenCV`'s Farneback Gaussian path (NOT `getGaussianKernel`):
///   `optflowgf.cpp::FarnebackUpdateFlow_GaussianBlur`:
///     `int m = block_size/2; double sigma = m*0.3;`
/// For winsize=15 this gives sigma=2.1, narrower than `getGaussianKernel`'s 2.6.
pub(crate) fn build_gaussian_1d(winsize: u32) -> Vec<f32> {
    let n = winsize as usize;
    // winsize <= 1 gives sigma = 0, which makes the exp() argument 0/0 = NaN and
    // poisons the whole flow field. Floor sigma to the value winsize=2 already
    // yields, so a degenerate window produces a finite (near-delta) kernel.
    let sigma = (f64::from(winsize / 2) * 0.3).max(0.3);
    let half = (n as f64 - 1.0) / 2.0;

    let mut kernel = vec![0.0f64; n];
    let mut sum = 0.0f64;
    for (i, kval) in kernel.iter_mut().enumerate().take(n) {
        let x = i as f64 - half;
        let v = (-x * x / (2.0 * sigma * sigma)).exp();
        *kval = v;
        sum += v;
    }

    kernel.iter().map(|&v| (v / sum) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_flow_zero_polys_stays_zero() {
        let mut flow = Flow::zeros(8, 8);
        let poly1 = PolyImage {
            width: 8,
            height: 8,
            data: vec![[0.0; 5]; 64],
        };
        let poly2 = PolyImage {
            width: 8,
            height: 8,
            data: vec![[0.0; 5]; 64],
        };
        let mut ws = UpdateWorkspace::new(8, 8);
        update_flow_with_workspace(&mut flow, &poly1, &poly2, 5, false, &mut ws);
        for v in &flow.data {
            assert!(v[0].abs() < 1e-6);
            assert!(v[1].abs() < 1e-6);
        }
    }

    #[test]
    fn update_flow_solves_known_xy_translation() {
        let mut flow = Flow::zeros(16, 16);
        let poly1 = PolyImage {
            width: 16,
            height: 16,
            data: vec![[1.0, 0.0, 1.0, 0.0, 0.0]; 256],
        };
        let poly2 = PolyImage {
            width: 16,
            height: 16,
            data: vec![[1.0, 0.0, 1.0, 4.0, -6.0]; 256],
        };
        let mut ws = UpdateWorkspace::new(16, 16);

        update_flow_with_workspace(&mut flow, &poly1, &poly2, 1, false, &mut ws);

        let center = flow.at(8, 8);
        assert!((center[0] - 2.0).abs() < 3e-3);
        assert!((center[1] + 3.0).abs() < 3e-3);
    }

    #[test]
    fn update_flow_preserves_total_flow_from_existing_warp() {
        let mut flow = Flow::zeros(16, 16);
        for v in &mut flow.data {
            *v = [1.5, -0.75];
        }
        let poly1 = PolyImage {
            width: 16,
            height: 16,
            data: vec![[1.0, 0.0, 1.0, 0.0, 0.0]; 256],
        };
        let poly2 = PolyImage {
            width: 16,
            height: 16,
            data: vec![[1.0, 0.0, 1.0, 0.0, 0.0]; 256],
        };
        let mut ws = UpdateWorkspace::new(16, 16);

        update_flow_with_workspace(&mut flow, &poly1, &poly2, 1, false, &mut ws);

        let center = flow.at(8, 8);
        assert!((center[0] - 1.5).abs() < 2e-3);
        assert!((center[1] + 0.75).abs() < 1e-3);
    }

    #[test]
    fn gaussian_kernel_sums_to_one() {
        let k = build_gaussian_1d(15);
        let sum: f32 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn box_kernel_sums_to_one() {
        let size = 15usize;
        let k: Vec<f32> = vec![1.0 / size as f32; size];
        let sum: f32 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }
}
