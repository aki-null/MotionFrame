# MotionFrame

**MotionFrame** is a Python tool designed to analyze flipbook images and generate motion vector textures for use with motion blend shaders.

The problem with flipbook textures is that the texture size becomes very large if you want smooth animation. The motion blend technique makes animation smoother with fewer frames by providing an extra motion vector texture.

## Sample Renders

| Fade Blend                                                                                         | Motion Blend                                                                                         |
| -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| ![Explosion Fade](https://github.com/user-attachments/assets/00159a36-f49e-4593-9221-0b4c80ca4113) | ![Explosion Motion](https://github.com/user-attachments/assets/239eac78-90b2-4018-bd08-68a8148c7642) |
| ![Smoke Fade](https://github.com/user-attachments/assets/e20742f8-35bc-403d-8da0-a7d2397d7e89)     | ![Smoke Motion](https://github.com/user-attachments/assets/c9e847f7-06a3-452e-b592-bc7e0b674782)     |
## Key Features

- **GUI Frontend**
- **Cross-Platform**:
  - Supports macOS, Windows, and likely Linux.
- **Skipped Frame Analysis**:
  - Enhances motion analysis precision with high frame rate input.
  - Analyzes every input frame and accumulates skipped frames into each motion vector frame.
  - Ideal for animations with fast movements, which are challenging for image-based motion analysis.
- **Motion Vector [Stagger Packing](https://realtimevfx.com/t/flipbook-texture-packing-atlas-super-pack-and-stagger-pack/5609)**:
  - Motion vector textures use 2 channels, but this may not be optimal for some platforms.
  - Packing them into 4 channels reduces texture size by half and compresses well with formats like ASTC.
- **Color Atlas Packing**
- **Motion Visualization**
- **Free and Open Source**

See the [reference shader implementation (MIT)](https://github.com/aki-null/UnityFlipbookMotionBlending) for shaders to render this.

## Installation

### Self Contained Binary

[Releases](https://github.com/aki-null/MotionFrame/releases)

### Setup

#### Windows

```bash
git clone https://github.com/aki-null/MotionFrame.git
cd MotionFrame
python -m venv .venv
source .venv/Scripts/activate
cd app
pip install -r requirements.txt
python3 MotionFrame.py
```

#### macOS

```bash
git clone https://github.com/aki-null/MotionFrame.git
cd MotionFrame
python -m venv .venv
source .venv/bin/activate
cd app
pip install -r requirements.txt
python3 MotionFrame.py
```

## Usage

![Main Window](https://github.com/user-attachments/assets/d44658e8-3a2d-4908-afb1-a22fbbed1fde)

- Drag and drop image sequence file
	- Folder works too
	- The tool tries its best to determine how your image sequence file names are formatted
- Configure options
	- Frame skips are important if your input frame count is not exactly the same as the desired output frames
	- The output motion vector quality improves if you can provide more input frames
- Generate
	- The tabs on the right displays various output
	- Visualization tab shows how the tool interpreted the motion in each frame with arrows
- Save to files
	- The output is in TGA
	- Color, motion vector, and JSON metadata are exported

## License

[GPL v3.0](https://www.gnu.org/licenses/gpl-3.0.txt)

```
MotionFrame
Copyright (C) 2024  Akihiro Noguchi

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```

- Downloading this tool and using the generated texture for your game doesn't contaminate your software with GPL v3.0 license. I would be happy if you credit me though :)

## Notes

- Make sure you import the motion vector texture as linear (non-sRGB) in your game engine
	- In Unity, uncheck sRGB (Color Textire) in texture settings
- The output includes a JSON file which contains some metadata that are useful for various shader parameters
	- You can take a look at the JSON file if you forget some properties of the output like the motion strength and total number of frames

## Appendix

Textures used to render the sample animations.

|Color Atlas|Motion Atlas|
|-----------|------------|
|![Explosion Color Atlas](https://github.com/user-attachments/assets/f68db0fe-9348-4ae3-bffa-cb2839ddc5a8)|![Explosion Motion Atlas](https://github.com/user-attachments/assets/5354ece8-0127-43a0-8b90-a9d358bab4e5)|
|![Smoke Color Atlas](https://github.com/user-attachments/assets/8bcb4457-e245-4f93-a1aa-4ec88763777f)|![Smoke Motion Atlas](https://github.com/user-attachments/assets/66e98695-c152-40ad-9aad-8b098f963d09)|

## References

- <https://www.klemenlozar.com/frame-blending-with-motion-vectors/>
