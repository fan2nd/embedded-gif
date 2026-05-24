#![no_std]
#![doc = include_str!("../README.md")]

mod color;
mod frame;
mod gif;
mod indexed;
mod rle;

pub use color::PaletteColor;
pub use embedded_gif_macros::{include_gif_frames, include_gif_indexed};
pub use embedded_graphics;
pub use frame::{DisposalMethod, GifFrame};
pub use gif::Gif;
pub use indexed::{IndexBitDepth, IndexedFrame, IndexedGif};
