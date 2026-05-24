use embedded_graphics::{draw_target::DrawTarget, geometry::Point, pixelcolor::PixelColor};

use crate::{color::PaletteColor, frame::GifFrame, rle::PaletteIndexColor};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IndexedGif<I, C>
where
    I: PixelColor + 'static,
    C: PaletteColor + 'static,
{
    frames: &'static [GifFrame<I>],
    palette: &'static [C],
    index: usize,
    elapsed_millis: u32,
}

impl<I, C> IndexedGif<I, C>
where
    I: PaletteIndexColor<C> + PixelColor + 'static,
    C: PaletteColor + 'static,
{
    pub const fn new(frames: &'static [GifFrame<I>], palette: &'static [C]) -> Self {
        Self {
            frames,
            palette,
            index: 0,
            elapsed_millis: 0,
        }
    }

    pub const fn frames(&self) -> &'static [GifFrame<I>] {
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

    pub fn current_frame(&self) -> Option<&GifFrame<I>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
    }

    pub fn advance(&mut self) -> Option<&GifFrame<I>> {
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
}
