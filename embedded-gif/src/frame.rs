use core::marker::PhantomData;

use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
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
    pixel_color: PhantomData<C>,
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
            pixel_color: PhantomData,
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
