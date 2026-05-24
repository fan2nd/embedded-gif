use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{BinaryColor, PixelColor, Rgb565, Rgb888},
    Pixel,
};

use crate::{
    frame::GifFrame,
    indexed::{IndexBitDepth, IndexedFrame},
};

const TOKEN_RAW: u8 = 0b1000_0000;
const TOKEN_REPEAT: u8 = 0b1100_0000;
const TOKEN_SKIP_MASK: u8 = 0b1000_0000;
const TOKEN_KIND_MASK: u8 = 0b1100_0000;
const TOKEN_LONG_LEN_MASK: u8 = 0b0111_1111;
const TOKEN_SHORT_LEN_MASK: u8 = 0b0011_1111;

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

pub(crate) fn draw_indexed_frame<C, D>(
    target: &mut D,
    frame: &IndexedFrame,
    palette: &[C],
    origin: Point,
) -> Result<(), D::Error>
where
    C: PixelColor + 'static,
    D: DrawTarget<Color = C>,
{
    let width = frame.size().width as usize;
    let height = frame.size().height as usize;
    let mut pixels = IndexedRlePixels::new(
        frame.data(),
        width.saturating_mul(height),
        frame.index_bit_depth(),
    );

    target.draw_iter(core::iter::from_fn(move || {
        let (pixel_index, index) = pixels.next_pixel()?;
        let x = pixel_index % width;
        let y = pixel_index / width;
        let color = *palette.get(index as usize)?;

        if y >= height {
            return None;
        }

        Some(Pixel(
            origin + frame.top_left() + Point::new(x as i32, y as i32),
            color,
        ))
    }))
}

struct IndexedRlePixels<'a> {
    data: &'a [u8],
    offset: usize,
    bit_offset: u8,
    cursor: usize,
    pixel_count: usize,
    raw_remaining: usize,
    repeat_remaining: usize,
    repeat_color: Option<u8>,
    bit_depth: IndexBitDepth,
}

impl<'a> IndexedRlePixels<'a> {
    fn new(data: &'a [u8], pixel_count: usize, bit_depth: IndexBitDepth) -> Self {
        Self {
            data,
            offset: 0,
            bit_offset: 0,
            cursor: 0,
            pixel_count,
            raw_remaining: 0,
            repeat_remaining: 0,
            repeat_color: None,
            bit_depth,
        }
    }

    fn next_pixel(&mut self) -> Option<(usize, u8)> {
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

            if self.repeat_remaining > 0 {
                let color = self.repeat_color?;
                let index = self.cursor;
                self.cursor += 1;
                self.repeat_remaining -= 1;
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

        if token & TOKEN_SKIP_MASK == 0 {
            let len = usize::from(token & TOKEN_LONG_LEN_MASK) + 1;
            self.cursor = self.cursor.saturating_add(len);
            return Some(());
        }

        let len = usize::from(token & TOKEN_SHORT_LEN_MASK) + 1;
        match token & TOKEN_KIND_MASK {
            TOKEN_RAW => self.raw_remaining = len,
            TOKEN_REPEAT => {
                self.repeat_color = Some(self.read_color()?);
                self.align_to_next_byte();
                self.repeat_remaining = len;
            }
            _ => return None,
        }

        Some(())
    }

    fn read_color(&mut self) -> Option<u8> {
        let mut value = 0u8;

        for _ in 0..self.bit_depth.bits() {
            let byte = *self.data.get(self.offset)?;
            let bit = (byte >> (7 - self.bit_offset)) & 1;
            value = (value << 1) | bit;

            self.bit_offset += 1;
            if self.bit_offset == 8 {
                self.offset += 1;
                self.bit_offset = 0;
            }
        }

        Some(value)
    }

    fn align_to_next_byte(&mut self) {
        if self.bit_offset != 0 {
            self.offset += 1;
            self.bit_offset = 0;
        }
    }
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
    repeat_remaining: usize,
    repeat_color: Option<C>,
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
            repeat_remaining: 0,
            repeat_color: None,
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

            if self.repeat_remaining > 0 {
                let color = self.repeat_color?;
                let index = self.cursor;
                self.cursor += 1;
                self.repeat_remaining -= 1;
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

        if token & TOKEN_SKIP_MASK == 0 {
            let len = usize::from(token & TOKEN_LONG_LEN_MASK) + 1;
            self.cursor = self.cursor.saturating_add(len);
            return Some(());
        }

        let len = usize::from(token & TOKEN_SHORT_LEN_MASK) + 1;
        match token & TOKEN_KIND_MASK {
            TOKEN_RAW => self.raw_remaining = len,
            TOKEN_REPEAT => {
                self.repeat_color = Some(self.read_color()?);
                self.align_to_next_byte();
                self.repeat_remaining = len;
            }
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
