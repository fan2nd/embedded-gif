#![no_std]
#![doc = include_str!("../README.md")]

pub use embedded_gif_macros::{include_complete_gif, include_raw_gif};
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
pub enum DisposalMethod {
    Any,
    Keep,
    Background,
    Previous,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CompleteGifFrame<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    image: GifImage<C, BO>,
    delay_centiseconds: u16,
}

impl<C, BO> CompleteGifFrame<C, BO>
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
pub struct RawGifFrame<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    image: GifImage<C, BO>,
    top_left: Point,
    delay_centiseconds: u16,
    disposal_method: DisposalMethod,
}

impl<C, BO> RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(
        image: GifImage<C, BO>,
        top_left: Point,
        delay_centiseconds: u16,
        disposal_method: DisposalMethod,
    ) -> Self {
        Self {
            image,
            top_left,
            delay_centiseconds,
            disposal_method,
        }
    }

    pub const fn image(&self) -> &GifImage<C, BO> {
        &self.image
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
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CompleteGif<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    frames: &'static [CompleteGifFrame<C, BO>],
    index: usize,
    elapsed_millis: u32,
}

impl<C, BO> CompleteGif<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(frames: &'static [CompleteGifFrame<C, BO>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
        }
    }

    pub const fn frames(&self) -> &'static [CompleteGifFrame<C, BO>] {
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

    pub fn current_frame(&self) -> Option<&CompleteGifFrame<C, BO>> {
        self.frames.get(self.index)
    }

    pub fn current_image(&self) -> Option<&GifImage<C, BO>> {
        self.current_frame().map(CompleteGifFrame::image)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
    }

    pub fn advance(&mut self) -> Option<&CompleteGifFrame<C, BO>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        tick_frame(
            &mut self.index,
            &mut self.elapsed_millis,
            self.frames,
            millis,
        )
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
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
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawGif<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    frames: &'static [RawGifFrame<C, BO>],
    index: usize,
    elapsed_millis: u32,
}

impl<C, BO> RawGif<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(frames: &'static [RawGifFrame<C, BO>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
        }
    }

    pub const fn frames(&self) -> &'static [RawGifFrame<C, BO>] {
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

    pub fn current_frame(&self) -> Option<&RawGifFrame<C, BO>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
    }

    pub fn advance(&mut self) -> Option<&RawGifFrame<C, BO>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        tick_frame(
            &mut self.index,
            &mut self.elapsed_millis,
            self.frames,
            millis,
        )
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current<D>(&self, target: &mut D, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if let Some(frame) = self.current_frame() {
            Image::new(frame.image(), origin + frame.top_left()).draw(target)
        } else {
            Ok(())
        }
    }
}

trait GifFrameTiming {
    fn delay_millis(&self) -> u32;
}

impl<C, BO> GifFrameTiming for CompleteGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    fn delay_millis(&self) -> u32 {
        self.delay_millis()
    }
}

impl<C, BO> GifFrameTiming for RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    fn delay_millis(&self) -> u32 {
        self.delay_millis()
    }
}

fn tick_frame<F>(
    index: &mut usize,
    elapsed_millis: &mut u32,
    frames: &'static [F],
    millis: u32,
) -> bool
where
    F: GifFrameTiming,
{
    if frames.len() <= 1 {
        return false;
    }

    *elapsed_millis = elapsed_millis.saturating_add(millis);

    let mut changed = false;
    while *elapsed_millis >= frames[*index].delay_millis().max(10) {
        *elapsed_millis -= frames[*index].delay_millis().max(10);
        *index = (*index + 1) % frames.len();
        changed = true;
    }

    changed
}
