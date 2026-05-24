use embedded_gif::embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    Pixel,
};
use embedded_gif::{
    include_gif, DisposalMethod, Gif, GifFrame, PaletteIndex1, PaletteIndex2, PaletteIndex4,
    PaletteIndex8,
};

static FRAMES: &[GifFrame] = include_gif!("tests/fixtures/two_frames.gif");
static RGB565_FRAMES: &[GifFrame<Rgb565>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);
static BINARY_FRAMES: &[GifFrame<BinaryColor>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = BinaryColor,
    dither = true
);
static PALETTE1_FRAMES: &[GifFrame<PaletteIndex1<Rgb565>>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = PaletteIndex1<Rgb565>
);
static PALETTE2_FRAMES: &[GifFrame<PaletteIndex2<Rgb565>>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = PaletteIndex2<Rgb565>
);
static PALETTE4_FRAMES: &[GifFrame<PaletteIndex4<Rgb565>>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = PaletteIndex4<Rgb565>
);
static PALETTE8_FRAMES: &[GifFrame<PaletteIndex8<Rgb565>>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = PaletteIndex8<Rgb565>
);

#[test]
fn embeds_raw_frames_with_rle_data() {
    assert_eq!(FRAMES.len(), 2);
    assert_eq!(FRAMES[0].size(), Size::new(1, 1));
    assert_eq!(FRAMES[0].data(), &[0x40, 0x00, 0x00, 0x00]);
    assert_eq!(FRAMES[0].top_left(), Point::new(0, 0));
    assert_eq!(FRAMES[0].delay_centiseconds(), 10);
    assert_eq!(FRAMES[0].delay_millis(), 100);
    assert_eq!(FRAMES[0].disposal_method(), DisposalMethod::Any);
}

#[test]
fn embeds_selected_pixel_formats() {
    assert_eq!(RGB565_FRAMES.len(), 2);
    assert_eq!(RGB565_FRAMES[0].size(), Size::new(1, 1));

    assert_eq!(BINARY_FRAMES.len(), 2);
    assert_eq!(BINARY_FRAMES[0].size(), Size::new(1, 1));

    assert_eq!(PALETTE1_FRAMES.len(), 2);
    assert_eq!(PALETTE1_FRAMES[0].data(), &[0x40, 0x00]);
    assert_eq!(PALETTE2_FRAMES.len(), 2);
    assert_eq!(PALETTE2_FRAMES[0].data(), &[0x40, 0x00]);
    assert_eq!(PALETTE4_FRAMES.len(), 2);
    assert_eq!(PALETTE4_FRAMES[0].data(), &[0x40, 0x00]);
    assert_eq!(PALETTE8_FRAMES.len(), 2);
    assert_eq!(PALETTE8_FRAMES[0].data(), &[0x40, 0x00]);
}

#[test]
fn manages_gif_timing() {
    let mut animation = Gif::new(FRAMES);

    assert_eq!(animation.len(), 2);
    assert_eq!(animation.frame_index(), 0);
    assert!(!animation.tick_millis(95));
    assert_eq!(animation.frame_index(), 0);
    assert!(animation.tick_millis(5));
    assert_eq!(animation.frame_index(), 1);
    assert!(animation.tick_centiseconds(10));
    assert_eq!(animation.frame_index(), 0);
}

#[test]
fn draws_packed_palette_indices() {
    static DATA: &[u8] = &[0x83, 0b0001_1011];
    static FRAMES: &[GifFrame<PaletteIndex2<Rgb565>>] = &[GifFrame::new(
        DATA,
        Size::new(4, 1),
        Point::zero(),
        10,
        DisposalMethod::Any,
    )];
    let animation = Gif::new(FRAMES);
    let mut target = PixelTarget::<PaletteIndex2<Rgb565>>::new(Size::new(4, 1));

    animation
        .draw_current_delta(&mut target, Point::zero())
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[
            Pixel(Point::new(0, 0), PaletteIndex2::<Rgb565>::new(0)),
            Pixel(Point::new(1, 0), PaletteIndex2::<Rgb565>::new(1)),
            Pixel(Point::new(2, 0), PaletteIndex2::<Rgb565>::new(2)),
            Pixel(Point::new(3, 0), PaletteIndex2::<Rgb565>::new(3)),
        ]
    );
}

struct PixelTarget<C: PixelColor> {
    size: Size,
    pixels: Vec<Pixel<C>>,
}

impl<C: PixelColor> PixelTarget<C> {
    fn new(size: Size) -> Self {
        Self {
            size,
            pixels: Vec::new(),
        }
    }
}

impl<C> DrawTarget for PixelTarget<C>
where
    C: PixelColor,
{
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.pixels.extend(pixels);
        Ok(())
    }
}

impl<C: PixelColor> OriginDimensions for PixelTarget<C> {
    fn size(&self) -> Size {
        self.size
    }
}
