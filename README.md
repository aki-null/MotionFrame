# MotionFrame

**MotionFrame** is a Rust tool for generating motion-vector textures for
flipbook animation. It analyzes an image sequence, computes optical flow,
accumulates sub-frame motion, and writes color and motion atlases that a
runtime shader can sample for smoother playback with fewer texture frames.

The project includes a desktop application, a command-line converter, and a web version.

Try it in your browser: [motionframe.aki-null.net](https://motionframe.aki-null.net)

## Sample Renders

| Fade Blend                                                                                         | Motion Blend                                                                                         |
| -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| ![Explosion Fade](https://github.com/user-attachments/assets/7ced1986-9f1e-4603-bcee-3e482959a23a) | ![Explosion Motion](https://github.com/user-attachments/assets/748efb31-e16e-4474-9818-e17ea926dc20) |
| ![Smoke Fade](https://github.com/user-attachments/assets/60760958-35c5-45b8-8096-3613640a6a37) | ![Smoke Motion](https://github.com/user-attachments/assets/9303e51c-5bf2-4210-8795-0f15cd3efe4b) |

## Key Features

- **GUI Frontend**
- **Batch Conversion CLI**
- **Cross-Platform**
  - Supports macOS, Windows, and Linux.
- **Skipped Frame Analysis**
  - Enhances motion analysis precision with high frame rate input.
  - Analyzes every input frame and accumulates skipped frames into each motion vector frame.
  - Useful for fast movement that is difficult for image-based motion analysis.
- **Motion Vector [Stagger Packing](https://realtimevfx.com/t/flipbook-texture-packing-atlas-super-pack-and-stagger-pack/5609)**
  - Motion vector textures use 2 channels, but this may not be optimal for some platforms.
  - Packing them into 4 channels reduces texture size by half and compresses well with formats like ASTC.
- **Color Atlas Packing**
- **Motion Visualization**
- **Free and Open Source**

See the [reference shader implementation (MIT)](https://github.com/aki-null/UnityFlipbookMotionBlending) for shaders to render this.

## Installation

### Self Contained Binary

[Releases](https://github.com/aki-null/MotionFrame/releases)

### Build from Source

Install the pinned Rust toolchain, then build:

```bash
git clone https://github.com/aki-null/MotionFrame.git
cd MotionFrame
cargo build --release --bin motionframe
```

The desktop binary is written to:

```text
target/release/motionframe
```

## Usage

Launch the desktop application:

```bash
cargo run --release --bin motionframe
```

In the desktop app:

![Desktop Screenshot](https://github.com/user-attachments/assets/94de16f4-842c-4660-a415-0ac973031251)

- Load frames: drag and drop an image sequence onto the window, or click Browse. A folder works (the tool infers the frame-number naming), as does a single atlas image (tile count is auto-detected).
- Configure options. Output frames sets the target output count, and Analyze skipped frames improves quality when the input has more frames than the output.
- Click Generate. The result appears across the tabs: Color, Motion, Visualization (motion as arrows), and Preview (animated GPU playback warped by the motion vectors).
- Click Save to export a color atlas and a motion vector atlas (TGA), plus JSON metadata.

Convert one image sequence or atlas from the command line:

```bash
cargo run --release --bin motionframe -- convert \
  --input frames/explosion \
  --output out/explosion \
  --output-count 64 \
  --layout auto
```

The CLI writes:

- `<prefix>_color_atlas.tga`
- `<prefix>_motion_atlas.tga`
- `<prefix>_meta.json`

Use `motionframe convert --help` for all conversion options.

## Migration from Python MotionFrame

MotionFrame 2.0 is a Rust rewrite of the original Python application. The
previous Python implementation is archived on the `legacy-python` branch and
the `python-final` tag.

### What changed

- Desktop application and CLI are built from Rust.
- Optical flow and atlas generation are implemented in the Rust engine.
- Runtime dependencies are no longer installed through Python.

### Compatibility

Existing image-sequence workflows should map to the Rust CLI and desktop app.
Output quality and performance are expected to improve. If a script depended
on Python internals, migrate it to the `motionframe` CLI.

### Legacy source

Use these refs for the old Python implementation:

- `legacy-python`
- `python-final`
- `v1.0.0-python-final`

## Verification

```bash
./scripts/verify.sh        # fmt + clippy + test + release build + web build + license check
./scripts/verify-quick.sh  # fmt + clippy + tests
./scripts/verify-parity.sh # flow + atlas pipeline parity tests
```

## License

[GPL v3.0](https://www.gnu.org/licenses/gpl-3.0.txt)

MotionFrame is licensed under GPL v3.0. Third-party notices and license texts
are listed in `THIRD-PARTY-LICENSES.md`.

## Notes

- Import the motion vector texture as linear (non-sRGB) in your game engine.
  - In Unity, uncheck "sRGB (Color Texture)" in the texture settings.
- The exported JSON holds metadata useful for shader parameters, such as motion strength and total frame count.

Downloading this tool and using generated textures for your game does not contaminate your software with GPL v3.0.

## Appendix

Generated atlases:

| Color Atlas                                                                                       | Motion Atlas                                                                                       |
| ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| ![Explosion Color Atlas](https://github.com/user-attachments/assets/a17d14d7-e2b4-4eea-b373-b77108511c70) | ![Explosion Motion Atlas](https://github.com/user-attachments/assets/a4e8d73b-6774-4d4d-810e-769538ae67b0) |
| ![Smoke Color Atlas](https://github.com/user-attachments/assets/776135ce-8dc8-4833-8b4e-1ec726a5f358)     | ![Smoke Motion Atlas](https://github.com/user-attachments/assets/6f16d0de-3f1b-4409-9486-e2214b235df4)     |

## References

- <https://www.klemenlozar.com/frame-blending-with-motion-vectors/>
