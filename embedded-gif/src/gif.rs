use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
    transform::Transform,
};

use crate::{
    frame::{DisposalMethod, GifFrame},
    rle::RleColor,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Gif<C = Rgb888>
where
    C: PixelColor + 'static,
{
    frames: &'static [GifFrame<C>],
    index: usize,
    elapsed_millis: u32,
    composited_index: Option<usize>,
    composited_origin: Option<Point>,
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
            composited_origin: None,
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
        self.composited_origin = None;
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
            crate::rle::draw_frame(target, frame, origin)
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

        if let Some(previous) = self.incremental_previous_frame(origin) {
            Self::dispose_frame(target, origin, background, previous)?;

            if let Some(frame) = self.current_frame() {
                crate::rle::draw_frame(target, frame, origin)?;
            }

            self.composited_index = Some(self.index);
            self.composited_origin = Some(origin);
            return Ok(());
        }

        if let Some(bounds) = Self::canvas_bounds(self.frames) {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            crate::rle::draw_frame(target, frame, origin)?;

            if frame_index != self.index {
                Self::dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        self.composited_origin = Some(origin);
        Ok(())
    }

    fn previous_index(&self) -> Option<usize> {
        match self.frames.len() {
            0 => None,
            _ if self.index == 0 => Some(self.frames.len() - 1),
            _ => Some(self.index - 1),
        }
    }

    fn incremental_previous_frame(&self, origin: Point) -> Option<&GifFrame<C>> {
        if self.index == 0 || self.composited_origin != Some(origin) {
            return None;
        }

        let previous_index = self.previous_index()?;

        if self.composited_index != Some(previous_index) {
            return None;
        }

        let previous = self.frames.get(previous_index)?;
        if previous.disposal_method() == DisposalMethod::Previous {
            None
        } else {
            Some(previous)
        }
    }

    fn dispose_frame<D>(
        target: &mut D,
        origin: Point,
        background: C,
        frame: &GifFrame<C>,
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

    fn canvas_bounds(frames: &[GifFrame<C>]) -> Option<Rectangle> {
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
}
