#![no_std]
#![doc = include_str!("../README.md")]

pub use embedded_gif_macros::{include_complete_gif, include_raw_gif};
pub use embedded_graphics;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    image::{Image, ImageRaw},
    iterator::raw::RawDataSlice,
    pixelcolor::{
        raw::{BigEndian, ByteOrder},
        PixelColor, Rgb888,
    },
    primitives::Rectangle,
    transform::Transform,
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
    composited_index: Option<usize>,
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
            composited_index: None,
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
        self.composited_index = None;
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

    pub fn draw_current_delta<D>(&self, target: &mut D, origin: Point) -> Result<(), D::Error>
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

    pub fn draw_current_composited<D>(
        &mut self,
        target: &mut D,
        origin: Point,
        background: C,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if self.frames.is_empty() {
            return Ok(());
        }

        if self.composited_index == previous_index(self.index, self.frames.len()) {
            if let Some(previous) = self
                .composited_index
                .and_then(|index| self.frames.get(index))
            {
                Self::dispose_frame(target, origin, background, previous)?;
            }

            if let Some(frame) = self.current_frame() {
                Self::draw_frame(target, origin, frame)?;
            }

            self.composited_index = Some(self.index);
            return Ok(());
        }

        if let Some(bounds) = self.canvas_bounds() {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            Self::draw_frame(target, origin, frame)?;

            if frame_index != self.index {
                Self::dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        Ok(())
    }

    fn draw_frame<D>(
        target: &mut D,
        origin: Point,
        frame: &RawGifFrame<C, BO>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        Image::new(frame.image(), origin + frame.top_left()).draw(target)
    }

    fn dispose_frame<D>(
        target: &mut D,
        origin: Point,
        background: C,
        frame: &RawGifFrame<C, BO>,
    ) -> Result<(), D::Error>
    where
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

    fn canvas_bounds(&self) -> Option<Rectangle> {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for frame in self.frames {
            let frame_bounds = frame.bounding_box();
            min_x = min_x.min(frame_bounds.top_left.x);
            min_y = min_y.min(frame_bounds.top_left.y);
            max_x = max_x.max(frame_bounds.top_left.x + frame_bounds.size.width as i32);
            max_y = max_y.max(frame_bounds.top_left.y + frame_bounds.size.height as i32);
        }

        if min_x == i32::MAX {
            None
        } else {
            Some(Rectangle::new(
                Point::new(min_x, min_y),
                embedded_graphics::geometry::Size::new(
                    (max_x - min_x) as u32,
                    (max_y - min_y) as u32,
                ),
            ))
        }
    }
}

impl<C, BO> RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.image.size())
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

fn previous_index(index: usize, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else if index == 0 {
        Some(len - 1)
    } else {
        Some(index - 1)
    }
}
