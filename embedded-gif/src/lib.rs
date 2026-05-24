#![no_std]
#![doc = include_str!("../README.md")]

pub use embedded_gif_macros::include_gif;
pub use embedded_graphics;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    image::{Image, ImageRaw},
    iterator::raw::RawDataSlice,
    pixelcolor::{
        raw::{BigEndian, ByteOrder},
        PixelColor, Rgb888,
    },
    Drawable,
};

pub type GifImage<C = Rgb888, BO = BigEndian> = ImageRaw<'static, C, BO>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GifFrame<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    image: GifImage<C, BO>,
    delay_centiseconds: u16,
}

impl<C, BO> GifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(image: GifImage<C, BO>, delay_centiseconds: u16) -> Self {
        Self {
            image,
            delay_centiseconds,
        }
    }

    pub const fn image(&self) -> &GifImage<C, BO> {
        &self.image
    }

    pub const fn delay_centiseconds(&self) -> u16 {
        self.delay_centiseconds
    }

    pub const fn delay_millis(&self) -> u32 {
        self.delay_centiseconds as u32 * 10
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GifAnimation<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    frames: &'static [GifFrame<C, BO>],
    index: usize,
    elapsed_millis: u32,
}

impl<C, BO> GifAnimation<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(frames: &'static [GifFrame<C, BO>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
        }
    }

    pub const fn frames(&self) -> &'static [GifFrame<C, BO>] {
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

    pub fn current_frame(&self) -> Option<&GifFrame<C, BO>> {
        self.frames.get(self.index)
    }

    pub fn current_image(&self) -> Option<&GifImage<C, BO>> {
        self.current_frame().map(GifFrame::image)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
    }

    pub fn advance(&mut self) -> Option<&GifFrame<C, BO>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn advance_by_millis(&mut self, millis: u32) -> bool {
        if self.frames.len() <= 1 {
            return false;
        }

        self.elapsed_millis = self.elapsed_millis.saturating_add(millis);

        let mut changed = false;
        while self.elapsed_millis >= self.current_delay_millis() {
            self.elapsed_millis -= self.current_delay_millis();
            self.index = (self.index + 1) % self.frames.len();
            changed = true;
        }

        changed
    }

    pub fn advance_by_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.advance_by_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if let Some(image) = self.current_image() {
            Image::new(image, position).draw(target)
        } else {
            Ok(())
        }
    }

    fn current_delay_millis(&self) -> u32 {
        self.current_frame()
            .map(|frame| frame.delay_millis().max(10))
            .unwrap_or(10)
    }
}
