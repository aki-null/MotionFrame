pub mod analyze;
pub mod atlas;
pub mod atlas_layout;
pub mod bidirectional;
pub mod encode;
pub mod output_detents;
pub mod pack;
pub mod run;
pub mod temporal;

pub use run::PackMode;

use std::path::PathBuf;

/// Progress messages from the pipeline worker to the UI.
///
/// The UI polls these per `update()` frame and requests a repaint on each
/// new message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Progress {
    /// Loading frames.
    Loading {
        /// Frames loaded so far.
        current: usize,
        /// Total frames to load.
        total: usize,
    },
    /// Pipeline stage reached.
    Stage {
        /// Human-readable stage name.
        name: String,
        /// Overall completion fraction [0.0, 1.0].
        fraction: f32,
    },
    /// Pipeline finished successfully.
    Done,
}

// Core data types: single-channel float image, RGBA8 image, 2-channel flow
// field. These are plain `Vec`-backed with explicit strides — no ndarray — to
// keep Farneback math and SIMD straightforward.

/// Single-channel f32 image, row-major.
pub struct ImageF32 {
    pub width: u32,
    pub height: u32,
    /// Single-channel, row-major.
    pub data: Vec<f32>,
}

/// RGBA u8 image, row-major.
#[derive(Clone, Debug)]
pub struct ImageRgba8 {
    pub width: u32,
    pub height: u32,
    /// 4 bytes per pixel, row-major.
    pub data: Vec<u8>,
}

/// 2-channel f32 flow field.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Flow {
    pub width: u32,
    pub height: u32,
    /// (dx, dy) per pixel.
    pub data: Vec<[f32; 2]>,
}

/// Loaded frame sequence.
pub struct Sequence {
    pub frames: Vec<ImageRgba8>,
    pub source_paths: Vec<PathBuf>,
}

/// Full pipeline options — one field per UI control.
///
/// Internal algorithm choices (Heun's integration, bicubic remap, non-loop
/// tail=zero) are hard-coded and have no corresponding options here.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)] // partial deserialization: missing/renamed fields fall back to Default per-field
#[allow(clippy::struct_excessive_bools)] // DESIGN specifies these bool fields verbatim
pub struct GenerateOptions {
    pub output_frames: u32,
    pub frame_skip: u32,
    /// Allows exact output frame counts by ignoring ending input frames.
    pub trim_tail_for_exact_output_count: bool,
    /// Per-tile output width in pixels. Total atlas width = `tile_pixel_width * atlas_dims.0`.
    pub tile_pixel_width: u32,
    pub atlas_dims: (u32, u32),
    pub stagger_pack: bool,
    pub analyze_skipped_frames: bool,
    /// Output color atlas with premultiplied alpha. Default off — most engines
    /// expect straight (non-premultiplied) RGBA. Internal flow analysis still
    /// uses a premultiplied copy regardless of this flag, so transparent
    /// pixels don't leak garbage RGB into the gradient.
    pub premultiplied_alpha: bool,
    // Farneback parameters are an internal pipeline tuning, not a user-facing
    // option. `skip` keeps them out of the persisted config so the values
    // always come from `FarnebackParams::pipeline_tuned()` at startup —
    // preventing stale per-machine config from silently driving different
    // results.
    #[serde(skip)]
    pub farneback: FarnebackParams,
    pub motion_vector_encoding: MotionVectorEncoding,
    pub is_loop: bool,
    pub halve_motion_vector: bool,
    /// Temporal smoothing of motion vectors across output frames.
    /// `0.0` = off (default), `1.0` = full 3-tap binomial filter.
    /// Values outside `[0, 1]` are clamped at apply time.
    #[serde(default)]
    pub temporal_smoothing: f32,
    pub extrude: u32,
    pub resize_algorithm: Interpolation,
    /// When `Some((cols, rows))`, the input is treated as an atlas image:
    /// the single source frame is decoded once and sliced into `cols × rows`
    /// tiles in row-major order (top-left origin) before running the
    /// pipeline. `None` runs sequence mode unchanged. UI enforces the same
    /// power-of-two set used by `atlas_dims`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub input_atlas_dims: Option<(u32, u32)>,
    /// Per-axis maximum size (in pixels) of either output atlas. Both color
    /// and motion atlases must satisfy `cols * tile_w <= output_atlas_max_dim`
    /// and `rows * tile_h <= output_atlas_max_dim`. Default is 8192 to match
    /// the WebGPU baseline `max_texture_dimension_2d`. UI exposes a power-of-
    /// two `ComboBox` in {1024, 2048, 4096, 8192}.
    #[serde(default = "default_output_atlas_max_dim")]
    pub output_atlas_max_dim: u32,
}

const fn default_output_atlas_max_dim() -> u32 {
    8192
}

/// Motion vector encoding format.
///
/// Both encodings short-circuit on `max_strength < 1e-8` (zero-motion guard
/// prevents NaN from `1/max_strength` on all-static or fully-masked sequences).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MotionVectorEncoding {
    R8G8Remap01,
    SidefxLabsR8G8,
}

impl MotionVectorEncoding {
    /// Wire format consumed by the preview shader's `mv_encoding` uniform.
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::R8G8Remap01 => 0,
            Self::SidefxLabsR8G8 => 1,
        }
    }
}

/// Resize interpolation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Interpolation {
    Nearest,
    Linear,
    Cubic,
    Lanczos,
}

/// Farneback optical flow parameters (mirrors `OpenCV`'s parameter semantics).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FarnebackParams {
    pub pyr_scale: f32,
    pub levels: u32,
    pub winsize: u32,
    pub iterations: u32,
    pub poly_n: u32,
    pub poly_sigma: f32,
    pub use_gaussian: bool,
}

// The baseline that the OpenCV reference fixtures in `flow_parity::*` were
// generated against, so `FarnebackParams::default()` is what those tests pin
// themselves to. Treat these values as frozen: changing any of them
// invalidates the golden `.flo` files.
//
// The running pipeline does NOT use this set — see
// `FarnebackParams::pipeline_tuned`.
impl Default for FarnebackParams {
    fn default() -> Self {
        Self {
            pyr_scale: 0.5,
            levels: 8,
            winsize: 15,
            iterations: 5,
            poly_n: 5,
            poly_sigma: 1.5,
            use_gaussian: true,
        }
    }
}

impl FarnebackParams {
    /// Configuration the running pipeline uses.
    ///
    /// Diverges from the `OpenCV` parity baseline in `winsize` only. `GenerateOptions`
    /// marks its `farneback` field `#[serde(skip)]`, so this is the single source of
    /// truth at runtime and no stale per-machine config can drive different results.
    ///
    /// `winsize: 31` was measured against held-out frames: build the motion vectors
    /// from frames i and i+2, interpolate frame i+1 the way the runtime shader does,
    /// and compare against the real frame i+1. The wider window beat 15 on every
    /// sequence tried — a low-contrast smoke loop, an explosion sheet, and the
    /// `explosion00` fixture — by 3 to 5 points of error reduction, and at every
    /// source resolution from 100 px to 512 px.
    ///
    /// The optimum is roughly constant in absolute pixels rather than proportional
    /// to source resolution, so this is a flat constant and not a derived one.
    /// Content-dependent optima span 21 to 45; 31 sits within half a point of the
    /// best value for every case measured. Cost is the separable smooth in
    /// `update_flow_with_workspace`, which is O(winsize) per pixel.
    pub fn pipeline_tuned() -> Self {
        Self {
            winsize: 31,
            ..Self::default()
        }
    }
}

// Quality-preferring defaults: analyze all sub-frames, cubic resize, 8×8
// atlas at 128 px per tile.
impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            output_frames: 64,
            frame_skip: 0,
            trim_tail_for_exact_output_count: false,
            tile_pixel_width: 128,
            atlas_dims: (8, 8),
            stagger_pack: false,
            analyze_skipped_frames: true,
            premultiplied_alpha: false, // most game engines expect straight alpha
            farneback: FarnebackParams::pipeline_tuned(),
            motion_vector_encoding: MotionVectorEncoding::R8G8Remap01,
            is_loop: false,
            halve_motion_vector: false,
            temporal_smoothing: 0.0,
            extrude: 0,
            resize_algorithm: Interpolation::Cubic,
            input_atlas_dims: None,
            output_atlas_max_dim: default_output_atlas_max_dim(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_tail_for_exact_output_count_defaults_off() {
        let opts = GenerateOptions::default();

        assert!(!opts.trim_tail_for_exact_output_count);
        assert_eq!(opts.output_frames, 64);
    }
}

impl ImageF32 {
    /// Create a zero-filled single-channel image.
    pub fn zeros(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; (width as usize) * (height as usize)],
        }
    }
}

impl ImageRgba8 {
    /// Create a zero-filled RGBA image.
    pub fn zeros(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width as usize) * (height as usize) * 4],
        }
    }
}

impl Flow {
    /// Create a zero-filled flow field.
    pub fn zeros(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![[0.0, 0.0]; (width as usize) * (height as usize)],
        }
    }

    /// Access flow at (x, y) immutably.
    ///
    /// # Panics
    ///
    /// Panics if `x >= self.width` or `y >= self.height`.
    pub fn at(&self, x: u32, y: u32) -> &[f32; 2] {
        &self.data[(y as usize) * (self.width as usize) + (x as usize)]
    }

    /// Access flow at (x, y) mutably.
    ///
    /// # Panics
    ///
    /// Panics if `x >= self.width` or `y >= self.height`.
    pub fn at_mut(&mut self, x: u32, y: u32) -> &mut [f32; 2] {
        &mut self.data[(y as usize) * (self.width as usize) + (x as usize)]
    }
}

// Pipeline error type — names every distinct failure mode so callers can
// present actionable messages. Worker panics are caught via
// `std::panic::catch_unwind` to surface as recoverable errors.
/// Pipeline error type.
#[derive(thiserror::Error, Debug)]
pub enum PipelineError {
    #[error("could not detect frame pattern in {0}")]
    PatternDetection(String),

    #[error("inconsistent frame dimensions: {actual:?} != expected {expected:?} ({path})")]
    DimensionMismatch {
        expected: (u32, u32),
        actual: (u32, u32),
        path: PathBuf,
    },

    #[error("inconsistent channel count: {actual} != expected {expected} ({path})")]
    ChannelMismatch {
        expected: u8,
        actual: u8,
        path: PathBuf,
    },

    #[error("failed to decode {0}: {1}")]
    DecodeFailed(PathBuf, String),

    #[error("failed to encode {0}: {1}")]
    EncodeFailed(PathBuf, String),

    #[error("failed to write metadata {0}: {1}")]
    MetadataWriteFailed(PathBuf, String),

    #[error("loaded {loaded} of {expected} frames")]
    PartialLoad { loaded: usize, expected: usize },

    #[error("{count} frame(s) won't fit; minimum frame_skip is {min_skip}")]
    AtlasOverflow { count: usize, min_skip: u32 },

    #[error("worker panicked: {0}")]
    WorkerPanic(String),

    #[error("need at least 2 frames; got {0}")]
    TooFewFrames(usize),

    #[error("generation cancelled by user")]
    Cancelled,

    #[error("{0}")]
    Other(String),

    #[error("atlas: {0}")]
    Atlas(#[from] crate::io::AtlasError),

    #[error("atlas mode requires exactly 1 source image; got {0}")]
    AtlasFrameCount(usize),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
