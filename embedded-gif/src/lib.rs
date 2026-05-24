#![no_std]
#![doc = include_str!("../README.md")]

pub use embedded_gif_macros::include_gif;
pub use embedded_graphics;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{BinaryColor, PixelColor, Rgb565, Rgb888},
    primitives::Rectangle,
    transform::Transform,
    Pixel,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisposalMethod {
    Any,
    Keep,
    Background,
    Previous,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GifFrame<C = Rgb888>
where
    C: PixelColor + 'static,
{
    data: &'static [u8],
    size: Size,
    top_left: Point,
    delay_centiseconds: u16,
    disposal_method: DisposalMethod,
    pixel_color: core::marker::PhantomData<C>,
}

impl<C> GifFrame<C>
where
    C: PixelColor + 'static,
{
    pub const fn new(
        data: &'static [u8],
        size: Size,
        top_left: Point,
        delay_centiseconds: u16,
        disposal_method: DisposalMethod,
    ) -> Self {
        Self {
            data,
            size,
            top_left,
            delay_centiseconds,
            disposal_method,
            pixel_color: core::marker::PhantomData,
        }
    }

    pub const fn data(&self) -> &'static [u8] {
        self.data
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub const fn top_left(&self) -> Point {
        self.top_left
    }

    pub const fn delay_centiseconds(&self) -> u16 {
        self.delay_centiseconds
    }

    pub const fn delay_millis(&self) -> u32 {
        self.delay_centiseconds as u32 * 10
    }

    pub const fn disposal_method(&self) -> DisposalMethod {
        self.disposal_method
    }

    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.size)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Gif<C = Rgb888>
where
    C: PixelColor + 'static,
{
    frames: &'static [GifFrame<C>],
    index: usize,
    elapsed_millis: u32,
    composited_index: Option<usize>,
}

impl<C> Gif<C>
where
    C: RleColor + PixelColor + 'static,
{
    pub const fn new(frames: &'static [GifFrame<C>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
            composited_index: None,
        }
    }

    pub const fn frames(&self) -> &'static [GifFrame<C>] {
        self.frames
    }

    pub const fn len(&self) -> usize {
        self.frames.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub const fn frame_index(&self) -> usize {
        self.index
    }

    pub fn current_frame(&self) -> Option<&GifFrame<C>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
        self.composited_index = None;
    }

    pub fn advance(&mut self) -> Option<&GifFrame<C>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        if self.frames.len() <= 1 {
            return false;
        }

        self.elapsed_millis = self.elapsed_millis.saturating_add(millis);

        let mut changed = false;
        while self.elapsed_millis >= self.frames[self.index].delay_millis().max(10) {
            self.elapsed_millis -= self.frames[self.index].delay_millis().max(10);
            self.index = (self.index + 1) % self.frames.len();
            changed = true;
        }

        changed
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current_delta<D>(&self, target: &mut D, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if let Some(frame) = self.current_frame() {
            draw_frame(target, frame, origin)
        } else {
            Ok(())
        }
    }

    pub fn draw_current_composited<D>(
        &mut self,
        target: &mut D,
        origin: Point,
        background: C,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if self.frames.is_empty() {
            return Ok(());
        }

        if self.composited_index == previous_index(self.index, self.frames.len()) {
            if let Some(previous) = self
                .composited_index
                .and_then(|index| self.frames.get(index))
            {
                dispose_frame(target, origin, background, previous)?;
            }

            if let Some(frame) = self.current_frame() {
                draw_frame(target, frame, origin)?;
            }

            self.composited_index = Some(self.index);
            return Ok(());
        }

        if let Some(bounds) = canvas_bounds(self.frames) {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            draw_frame(target, frame, origin)?;

            if frame_index != self.index {
                dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        Ok(())
    }
}

fn previous_index(index: usize, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else if index == 0 {
        Some(len - 1)
    } else {
        Some(index - 1)
    }
}

fn dispose_frame<C, D>(
    target: &mut D,
    origin: Point,
    background: C,
    frame: &GifFrame<C>,
) -> Result<(), D::Error>
where
    C: PixelColor + 'static,
    D: DrawTarget<Color = C>,
{
    match frame.disposal_method() {
        DisposalMethod::Any | DisposalMethod::Keep => Ok(()),
        DisposalMethod::Background => {
            target.fill_solid(&frame.bounding_box().translate(origin), background)
        }
        DisposalMethod::Previous => Ok(()),
    }
}

fn canvas_bounds<C>(frames: &[GifFrame<C>]) -> Option<Rectangle>
where
    C: PixelColor + 'static,
{
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for frame in frames {
        let bounds = frame.bounding_box();
        min_x = min_x.min(bounds.top_left.x);
        min_y = min_y.min(bounds.top_left.y);
        max_x = max_x.max(bounds.top_left.x + bounds.size.width as i32);
        max_y = max_y.max(bounds.top_left.y + bounds.size.height as i32);
    }

    if min_x == i32::MAX {
        None
    } else {
        Some(Rectangle::new(
            Point::new(min_x, min_y),
            Size::new((max_x - min_x) as u32, (max_y - min_y) as u32),
        ))
    }
}

fn draw_frame<C, D>(target: &mut D, frame: &GifFrame<C>, origin: Point) -> Result<(), D::Error>
where
    C: RleColor + PixelColor + 'static,
    D: DrawTarget<Color = C>,
{
    let width = frame.size().width as usize;
    let height = frame.size().height as usize;
    let pixel_count = width.saturating_mul(height);
    let mut reader = RlePixels::<C>::new(frame.data(), pixel_count);

    target.draw_iter(core::iter::from_fn(move || {
        let (pixel_index, color) = reader.next_pixel()?;
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

const TOKEN_SKIP: u8 = 0b0000_0000;
const TOKEN_SOLID: u8 = 0b0100_0000;
const TOKEN_RAW: u8 = 0b1000_0000;
const TOKEN_RESERVED: u8 = 0b1100_0000;
const TOKEN_KIND_MASK: u8 = 0b1100_0000;
const TOKEN_LEN_MASK: u8 = 0b0011_1111;

#[doc(hidden)]
pub trait RleColor: Copy {
    const BYTES_PER_PIXEL: usize;

    fn read(bytes: &[u8]) -> Option<Self>;
}

impl RleColor for Rgb888 {
    const BYTES_PER_PIXEL: usize = 3;

    fn read(bytes: &[u8]) -> Option<Self> {
        Some(Self::new(*bytes.first()?, *bytes.get(1)?, *bytes.get(2)?))
    }
}

impl RleColor for Rgb565 {
    const BYTES_PER_PIXEL: usize = 2;

    fn read(bytes: &[u8]) -> Option<Self> {
        let value = u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]);
        let red = ((value >> 11) & 0x1f) as u8;
        let green = ((value >> 5) & 0x3f) as u8;
        let blue = (value & 0x1f) as u8;
        Some(Self::new(red, green, blue))
    }
}

impl RleColor for BinaryColor {
    const BYTES_PER_PIXEL: usize = 1;

    fn read(bytes: &[u8]) -> Option<Self> {
        match *bytes.first()? {
            0 => Some(Self::Off),
            _ => Some(Self::On),
        }
    }
}

struct RlePixels<'a, C>
where
    C: RleColor,
{
    data: &'a [u8],
    offset: usize,
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
                return Some((index, color));
            }

            if self.solid_remaining > 0 {
                let color = self.solid_color?;
                let index = self.cursor;
                self.cursor += 1;
                self.solid_remaining -= 1;
                return Some((index, color));
            }

            let token = *self.data.get(self.offset)?;
            self.offset += 1;
            let len = usize::from(token & TOKEN_LEN_MASK) + 1;

            match token & TOKEN_KIND_MASK {
                TOKEN_SKIP => {
                    self.cursor = self.cursor.saturating_add(len);
                }
                TOKEN_SOLID => {
                    self.solid_color = Some(self.read_color()?);
                    self.solid_remaining = len;
                }
                TOKEN_RAW => {
                    self.raw_remaining = len;
                }
                TOKEN_RESERVED => return None,
                _ => return None,
            }
        }
    }

    fn read_color(&mut self) -> Option<C> {
        let end = self.offset.checked_add(C::BYTES_PER_PIXEL)?;
        let color = C::read(self.data.get(self.offset..end)?)?;
        self.offset = end;
        Some(color)
    }
}
