#![no_std]
#![doc = include_str!("../README.md")]

mod color;
mod frame;
mod gif;
mod rle;

pub use color::{PaletteIndex1, PaletteIndex2, PaletteIndex4, PaletteIndex8};
pub use embedded_gif_macros::include_gif_frames;
pub use embedded_graphics;
pub use frame::{DisposalMethod, GifFrame};
pub use gif::Gif;
