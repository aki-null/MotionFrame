//! Sources narrower than the Farneback window must not panic.
//!
//! `sep_conv_horiz_row` derives its SIMD/scalar split from the kernel
//! half-width. Unclamped, that split exceeds the row length for a row shorter
//! than `winsize / 2` and the scalar-tail slice panics — a crash, in release,
//! on ordinary decoded input. The threshold tracks `winsize`, so raising the
//! pipeline's window widened the affected range from 7 px to 15 px.

use motionframe_engine::io::InMemoryFrames;
use motionframe_engine::pipeline::run::run_pipeline;
use motionframe_engine::pipeline::{FarnebackParams, GenerateOptions, ImageRgba8, Progress};

fn frames(n: u32, w: u32, h: u32) -> Vec<ImageRgba8> {
    (0..n)
        .map(|i| {
            let mut data = vec![0u8; (w * h * 4) as usize];
            for y in (2 + i)..(6 + i).min(h) {
                for x in 1..(w - 1).max(1) {
                    let idx = ((y * w + x) * 4) as usize;
                    data[idx] = 200;
                    data[idx + 1] = 180;
                    data[idx + 2] = 160;
                    data[idx + 3] = 255;
                }
            }
            ImageRgba8 {
                width: w,
                height: h,
                data,
            }
        })
        .collect()
}

#[test]
fn narrow_sources_do_not_panic() {
    for w in [1u32, 2, 3, 5, 8, 12, 14, 15, 16, 33] {
        for winsize in [15u32, 31] {
            let src = InMemoryFrames::new(frames(4, w, 64)).expect("frames");
            let opts = GenerateOptions {
                output_frames: 4,
                tile_pixel_width: w,
                atlas_dims: (2, 2),
                farneback: FarnebackParams {
                    winsize,
                    ..FarnebackParams::default()
                },
                ..GenerateOptions::default()
            };
            let result = run_pipeline(&src, &opts, &|_: Progress| {}, &|| false);
            assert!(
                result.is_ok(),
                "width {w} winsize {winsize} failed: {:?}",
                result.err()
            );
        }
    }
}
