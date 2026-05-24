use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{BinaryColor, PixelColor, Rgb565, Rgb888},
    Pixel,
};

use crate::{
    color::{PaletteIndex1, PaletteIndex2, PaletteIndex4, PaletteIndex8},
    frame::GifFrame,
};

const TOKEN_SKIP: u8 = 0b0000_0000;
const TOKEN_SOLID: u8 = 0b0100_0000;
const TOKEN_RAW: u8 = 0b1000_0000;
const TOKEN_RESERVED: u8 = 0b1100_0000;
const TOKEN_KIND_MASK: u8 = 0b1100_0000;
const TOKEN_LEN_MASK: u8 = 0b0011_1111;

#[doc(hidden)]
pub trait RleColor: Copy {
    const BITS_PER_PIXEL: usize;

    fn read(value: u32) -> Option<Self>;
}

impl RleColor for Rgb888 {
    const BITS_PER_PIXEL: usize = 24;

    fn read(value: u32) -> Option<Self> {
        Some(Self::new(
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
        ))
    }
}

impl RleColor for Rgb565 {
    const BITS_PER_PIXEL: usize = 16;

    fn read(value: u32) -> Option<Self> {
        let red = ((value >> 11) & 0x1f) as u8;
        let green = ((value >> 5) & 0x3f) as u8;
        let blue = (value & 0x1f) as u8;
        Some(Self::new(red, green, blue))
    }
}

impl RleColor for BinaryColor {
    const BITS_PER_PIXEL: usize = 1;

    fn read(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            _ => Some(Self::On),
        }
    }
}

macro_rules! impl_rle_palette_index {
    ($name:ident, $bits:expr) => {
        impl<C> RleColor for $name<C>
        where
            C: PixelColor,
        {
            const BITS_PER_PIXEL: usize = $bits;

            fn read(value: u32) -> Option<Self> {
                Some(Self::new(value as u8))
            }
        }
    };
}

impl_rle_palette_index!(PaletteIndex1, 1);
impl_rle_palette_index!(PaletteIndex2, 2);
impl_rle_palette_index!(PaletteIndex4, 4);
impl_rle_palette_index!(PaletteIndex8, 8);

pub(crate) fn draw_frame<C, D>(
    target: &mut D,
    frame: &GifFrame<C>,
    origin: Point,
) -> Result<(), D::Error>
where
    C: RleColor + PixelColor + 'static,
    D: DrawTarget<Color = C>,
{
    let width = frame.size().width as usize;
    let height = frame.size().height as usize;
    let mut pixels = RlePixels::<C>::new(frame.data(), width.saturating_mul(height));

    target.draw_iter(core::iter::from_fn(move || {
        let (pixel_index, color) = pixels.next_pixel()?;
        let x = pixel_index % width;
        let y = pixel_index / width;

        if y >= height {
            return None;
        }

        Some(Pixel(
            origin + frame.top_left() + Point::new(x as i32, y as i32),
            color,
        ))
    }))
}

struct RlePixels<'a, C>
where
    C: RleColor,
{
    data: &'a [u8],
    offset: usize,
    bit_offset: u8,
    cursor: usize,
    pixel_count: usize,
    raw_remaining: usize,
    solid_remaining: usize,
    solid_color: Option<C>,
}

impl<'a, C> RlePixels<'a, C>
where
    C: RleColor,
{
    fn new(data: &'a [u8], pixel_count: usize) -> Self {
        Self {
            data,
            offset: 0,
            bit_offset: 0,
            cursor: 0,
            pixel_count,
            raw_remaining: 0,
            solid_remaining: 0,
            solid_color: None,
        }
    }

    fn next_pixel(&mut self) -> Option<(usize, C)> {
        loop {
            if self.cursor >= self.pixel_count {
                return None;
            }

            if self.raw_remaining > 0 {
                let color = self.read_color()?;
                let index = self.cursor;
                self.cursor += 1;
                self.raw_remaining -= 1;
                if self.raw_remaining == 0 {
                    self.align_to_next_byte();
                }
                return Some((index, color));
            }

            if self.solid_remaining > 0 {
                let color = self.solid_color?;
                let index = self.cursor;
                self.cursor += 1;
                self.solid_remaining -= 1;
                return Some((index, color));
            }

            if self.bit_offset != 0 {
                return None;
            }

            self.read_token()?;
        }
    }

    fn read_token(&mut self) -> Option<()> {
        let token = *self.data.get(self.offset)?;
        self.offset += 1;
        let len = usize::from(token & TOKEN_LEN_MASK) + 1;

        match token & TOKEN_KIND_MASK {
            TOKEN_SKIP => self.cursor = self.cursor.saturating_add(len),
            TOKEN_SOLID => {
                self.solid_color = Some(self.read_color()?);
                self.align_to_next_byte();
                self.solid_remaining = len;
            }
            TOKEN_RAW => self.raw_remaining = len,
            TOKEN_RESERVED => return None,
            _ => return None,
        }

        Some(())
    }

    fn read_color(&mut self) -> Option<C> {
        let mut value = 0u32;

        for _ in 0..C::BITS_PER_PIXEL {
            let byte = *self.data.get(self.offset)?;
            let bit = (byte >> (7 - self.bit_offset)) & 1;
            value = (value << 1) | u32::from(bit);

            self.bit_offset += 1;
            if self.bit_offset == 8 {
                self.bit_offset = 0;
                self.offset += 1;
            }
        }

        C::read(value)
    }

    fn align_to_next_byte(&mut self) {
        if self.bit_offset != 0 {
            self.bit_offset = 0;
            self.offset += 1;
        }
    }
}
