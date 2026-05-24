use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    primitives::Rectangle,
    transform::Transform,
};

use crate::{color::PaletteColor, frame::DisposalMethod};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IndexBitDepth {
    One,
    Two,
    Four,
    Eight,
}

impl IndexBitDepth {
    pub const fn bits(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IndexedFrame {
    data: &'static [u8],
    size: Size,
    top_left: Point,
    delay_centiseconds: u16,
    disposal_method: DisposalMethod,
    index_bit_depth: IndexBitDepth,
}

impl IndexedFrame {
    pub const fn new(
        data: &'static [u8],
        size: Size,
        top_left: Point,
        delay_centiseconds: u16,
        disposal_method: DisposalMethod,
        index_bit_depth: IndexBitDepth,
    ) -> Self {
        Self {
            data,
            size,
            top_left,
            delay_centiseconds,
            disposal_method,
            index_bit_depth,
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

    pub const fn index_bit_depth(&self) -> IndexBitDepth {
        self.index_bit_depth
    }

    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.size)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IndexedGif<C>
where
    C: PaletteColor + 'static,
{
    frames: &'static [IndexedFrame],
    palette: &'static [C],
    index: usize,
    elapsed_millis: u32,
    composited_index: Option<usize>,
}

impl<C> IndexedGif<C>
where
    C: PaletteColor + 'static,
{
    pub const fn new(frames: &'static [IndexedFrame], palette: &'static [C]) -> Self {
        Self {
            frames,
            palette,
            index: 0,
            elapsed_millis: 0,
            composited_index: None,
        }
    }

    pub const fn frames(&self) -> &'static [IndexedFrame] {
        self.frames
    }

    pub const fn palette(&self) -> &'static [C] {
        self.palette
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

    pub fn current_frame(&self) -> Option<&IndexedFrame> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
        self.composited_index = None;
    }

    pub fn advance(&mut self) -> Option<&IndexedFrame> {
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
            crate::rle::draw_indexed_frame(target, frame, self.palette, origin)
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

        if self.composited_index == self.previous_index() {
            if let Some(previous) = self
                .composited_index
                .and_then(|index| self.frames.get(index))
            {
                Self::dispose_frame(target, origin, background, previous)?;
            }

            if let Some(frame) = self.current_frame() {
                crate::rle::draw_indexed_frame(target, frame, self.palette, origin)?;
            }

            self.composited_index = Some(self.index);
            return Ok(());
        }

        if let Some(bounds) = Self::canvas_bounds(self.frames) {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            crate::rle::draw_indexed_frame(target, frame, self.palette, origin)?;

            if frame_index != self.index {
                Self::dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        Ok(())
    }

    fn previous_index(&self) -> Option<usize> {
        match self.frames.len() {
            0 => None,
            _ if self.index == 0 => Some(self.frames.len() - 1),
            _ => Some(self.index - 1),
        }
    }

    fn dispose_frame<D>(
        target: &mut D,
        origin: Point,
        background: C,
        frame: &IndexedFrame,
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

    fn canvas_bounds(frames: &[IndexedFrame]) -> Option<Rectangle> {
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
