use criterion::{criterion_group, criterion_main, Criterion};
use motionframe_engine::flow::farneback::farneback;
use motionframe_engine::pipeline::{FarnebackParams, ImageF32};

/// Generate a synthetic gradient image of given dimensions.
fn synthetic_gradient(width: u32, height: u32) -> ImageF32 {
    let mut data = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            data.push((y as f32).mul_add(0.5, x as f32) / (width as f32));
        }
    }
    ImageF32 {
        width,
        height,
        data,
    }
}

/// Shift an image by (dx, dy) pixels using bilinear interpolation.
fn shift(img: &ImageF32, dx: f32, dy: f32) -> ImageF32 {
    let mut out = ImageF32::zeros(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let sx = x as f32 - dx;
            let sy = y as f32 - dy;
            if sx >= 0.0 && sx < (img.width - 1) as f32 && sy >= 0.0 && sy < (img.height - 1) as f32
            {
                let x0 = sx as u32;
                let y0 = sy as u32;
                let fx = sx - x0 as f32;
                let fy = sy - y0 as f32;
                let idx00 = (y0 * img.width + x0) as usize;
                let idx01 = idx00 + 1;
                let idx10 = idx00 + img.width as usize;
                let idx11 = idx10 + 1;
                let v = (img.data[idx00] * (1.0 - fx))
                    .mul_add(1.0 - fy, img.data[idx01] * fx * (1.0 - fy))
                    + (img.data[idx10] * (1.0 - fx)).mul_add(fy, img.data[idx11] * fx * fy);
                out.data[(y * img.width + x) as usize] = v;
            }
        }
    }
    out
}

fn bench_farneback_256(c: &mut Criterion) {
    let g1 = synthetic_gradient(256, 256);
    let g2 = shift(&g1, 3.0, 2.0);
    let params = FarnebackParams::default();
    c.bench_function("farneback 256x256", |b| {
        b.iter(|| farneback(&g1, &g2, &params));
    });
}

fn bench_farneback_400(c: &mut Criterion) {
    let g1 = synthetic_gradient(400, 400);
    let g2 = shift(&g1, 5.0, 3.0);
    let params = FarnebackParams::default();
    c.bench_function("farneback 400x400", |b| {
        b.iter(|| farneback(&g1, &g2, &params));
    });
}

/// The configuration the pipeline actually runs (`winsize: 31`), tracked
/// separately from the frozen `default()` parity baseline above.
///
/// The separable smooth in `update_flow_with_workspace` is O(winsize) per pixel
/// per component per iteration per level and dominates the inner loop, so a
/// regression there is invisible in the `default()` cases.
fn bench_farneback_400_pipeline_tuned(c: &mut Criterion) {
    let g1 = synthetic_gradient(400, 400);
    let g2 = shift(&g1, 5.0, 3.0);
    let params = FarnebackParams::pipeline_tuned();
    c.bench_function("farneback 400x400 pipeline-tuned", |b| {
        b.iter(|| farneback(&g1, &g2, &params));
    });
}

fn bench_pipeline_e2e(c: &mut Criterion) {
    use motionframe_engine::io::InMemoryFrames;
    use motionframe_engine::pipeline::run::run_pipeline;
    use motionframe_engine::pipeline::{GenerateOptions, ImageRgba8};

    // Generate 32 synthetic RGBA frames at 128x128 with a moving gradient
    let w = 128u32;
    let h = 128u32;
    let n_frames = 32;
    let frames: Vec<ImageRgba8> = (0..n_frames)
        .map(|f| {
            let mut data = vec![0u8; (w * h * 4) as usize];
            let offset = f as f32 * 2.0;
            for y in 0..h {
                for x in 0..w {
                    let idx = ((y * w + x) as usize) * 4;
                    let v = (((x as f32 + offset) / w as f32) * 255.0).min(255.0) as u8;
                    data[idx] = v;
                    data[idx + 1] = ((y as f32 / h as f32) * 255.0) as u8;
                    data[idx + 2] = 0;
                    data[idx + 3] = 255;
                }
            }
            ImageRgba8 {
                width: w,
                height: h,
                data,
            }
        })
        .collect();

    let opts = GenerateOptions {
        frame_skip: 2,
        tile_pixel_width: 128,
        atlas_dims: (4, 4),
        ..GenerateOptions::default()
    };

    let source = InMemoryFrames::new(frames).unwrap();
    let progress_fn = |_p: motionframe_engine::pipeline::Progress| {};
    let cancel_fn = || false;

    c.bench_function("pipeline e2e 32×128×128 skip2", |b| {
        b.iter(|| run_pipeline(&source, &opts, &progress_fn, &cancel_fn).unwrap());
    });
}

criterion_group!(
    benches,
    bench_farneback_256,
    bench_farneback_400,
    bench_farneback_400_pipeline_tuned,
    bench_pipeline_e2e
);
criterion_main!(benches);
