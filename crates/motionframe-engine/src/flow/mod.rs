//! Farneback optical flow — pure Rust implementation.
//!
//! Derived from `OpenCV`'s `optflowgf.cpp` (Farneback dense optical flow).
//! See `THIRD-PARTY-LICENSES.md` for license details.

pub mod contrast;
pub mod farneback;
pub mod poly;
pub mod pyramid;
pub mod update;

#[cfg(test)]
mod tests;
