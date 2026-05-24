use embedded_gif::embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    Pixel,
};
use embedded_gif::{
    include_gif_frames, include_gif_indexed, DisposalMethod, Gif, GifFrame, IndexBitDepth,
    IndexedFrame, IndexedGif,
};

static FRAMES: &[GifFrame] = include_gif_frames!("tests/fixtures/two_frames.gif");
static RGB565_FRAMES: &[GifFrame<Rgb565>] =
    include_gif_frames!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);
static BINARY_FRAMES: &[GifFrame<BinaryColor>] = include_gif_frames!(
    "tests/fixtures/two_frames.gif",
    pixel_format = BinaryColor,
    dither = true
);
static INDEXED_GIF: IndexedGif<Rgb565> =
    include_gif_indexed!("tests/fixtures/two_frames.gif", color = Rgb565);

#[test]
fn embeds_raw_frames_with_rle_data() {
    assert_eq!(FRAMES.len(), 2);
    assert_eq!(FRAMES[0].size(), Size::new(1, 1));
    assert_eq!(FRAMES[0].data(), &[0xc0, 0x00, 0x00, 0x00]);
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
}

#[test]
fn embeds_indexed_gif_with_palette() {
    assert_eq!(INDEXED_GIF.len(), 2);
    assert!(!INDEXED_GIF.palette().is_empty());
    assert_eq!(
        INDEXED_GIF.frames()[0].index_bit_depth(),
        IndexBitDepth::One
    );
    assert_eq!(INDEXED_GIF.frames()[0].data(), &[0xc0, 0x00]);
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
fn draws_dynamically_packed_palette_indices() {
    static DATA: &[u8] = &[0x83, 0b0001_1011];
    static FRAMES: &[IndexedFrame] = &[IndexedFrame::new(
        DATA,
        Size::new(4, 1),
        Point::zero(),
        10,
        DisposalMethod::Any,
        IndexBitDepth::Two,
    )];
    static PALETTE: &[Rgb565] = &[Rgb565::BLACK, Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE];
    let animation = IndexedGif::new(FRAMES, PALETTE);
    let mut target = PixelTarget::<Rgb565>::new(Size::new(4, 1));

    animation
        .draw_current_delta(&mut target, Point::zero())
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[
            Pixel(Point::new(0, 0), Rgb565::BLACK),
            Pixel(Point::new(1, 0), Rgb565::RED),
            Pixel(Point::new(2, 0), Rgb565::GREEN),
            Pixel(Point::new(3, 0), Rgb565::BLUE),
        ]
    );
}

#[test]
fn draws_local_frame_offsets_after_transparent_skip() {
    static DATA: &[u8] = &[0x04, 0xc0, 0xff, 0x00, 0x00];
    static FRAMES: &[GifFrame] = &[GifFrame::new(
        DATA,
        Size::new(3, 2),
        Point::new(2, 1),
        10,
        DisposalMethod::Any,
    )];
    let animation = Gif::new(FRAMES);
    let mut target = PixelTarget::new(Size::new(6, 4));

    animation
        .draw_current_delta(&mut target, Point::zero())
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[Pixel(
            Point::new(4, 2),
            embedded_gif::embedded_graphics::pixelcolor::Rgb888::RED
        )]
    );
}

#[test]
fn draws_indexed_local_frame_offsets_after_transparent_skip() {
    static DATA: &[u8] = &[0x04, 0xc0, 0x40];
    static FRAMES: &[IndexedFrame] = &[IndexedFrame::new(
        DATA,
        Size::new(3, 2),
        Point::new(2, 1),
        10,
        DisposalMethod::Any,
        IndexBitDepth::Two,
    )];
    static PALETTE: &[Rgb565] = &[Rgb565::BLACK, Rgb565::RED];
    let animation = IndexedGif::new(FRAMES, PALETTE);
    let mut target = PixelTarget::<Rgb565>::new(Size::new(6, 4));

    animation
        .draw_current_delta(&mut target, Point::zero())
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[Pixel(Point::new(4, 2), Rgb565::RED)]
    );
}

#[test]
fn indexed_composited_drawing_disposes_background() {
    static FRAME0_DATA: &[u8] = &[0xc1, 0x40];
    static FRAME1_DATA: &[u8] = &[0xc0, 0x80];
    static FRAMES: &[IndexedFrame] = &[
        IndexedFrame::new(
            FRAME0_DATA,
            Size::new(2, 1),
            Point::zero(),
            10,
            DisposalMethod::Background,
            IndexBitDepth::Two,
        ),
        IndexedFrame::new(
            FRAME1_DATA,
            Size::new(1, 1),
            Point::zero(),
            10,
            DisposalMethod::Any,
            IndexBitDepth::Two,
        ),
    ];
    static PALETTE: &[Rgb565] = &[Rgb565::BLACK, Rgb565::RED, Rgb565::GREEN];
    let mut animation = IndexedGif::new(FRAMES, PALETTE);
    let mut target = PixelTarget::<Rgb565>::new(Size::new(2, 1));

    animation
        .draw_current_composited(&mut target, Point::zero(), Rgb565::BLACK)
        .unwrap();
    animation.advance();
    animation
        .draw_current_composited(&mut target, Point::zero(), Rgb565::BLACK)
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[
            Pixel(Point::new(0, 0), Rgb565::BLACK),
            Pixel(Point::new(1, 0), Rgb565::BLACK),
            Pixel(Point::new(0, 0), Rgb565::RED),
            Pixel(Point::new(1, 0), Rgb565::RED),
            Pixel(Point::new(0, 0), Rgb565::BLACK),
            Pixel(Point::new(1, 0), Rgb565::BLACK),
            Pixel(Point::new(0, 0), Rgb565::GREEN),
        ]
    );
}

#[test]
fn composited_drawing_replays_after_previous_disposal() {
    static FRAME0_DATA: &[u8] = &[0xc0, 0xff, 0x00, 0x00];
    static FRAME1_DATA: &[u8] = &[0xc0, 0x00, 0xff, 0x00];
    static FRAME2_DATA: &[u8] = &[0xc0, 0x00, 0x00, 0xff];
    static FRAMES: &[GifFrame] = &[
        GifFrame::new(
            FRAME0_DATA,
            Size::new(1, 1),
            Point::new(0, 0),
            10,
            DisposalMethod::Keep,
        ),
        GifFrame::new(
            FRAME1_DATA,
            Size::new(1, 1),
            Point::new(2, 0),
            10,
            DisposalMethod::Previous,
        ),
        GifFrame::new(
            FRAME2_DATA,
            Size::new(1, 1),
            Point::new(1, 0),
            10,
            DisposalMethod::Keep,
        ),
    ];
    let mut animation = Gif::new(FRAMES);
    let mut target = PixelTarget::new(Size::new(3, 1));

    animation
        .draw_current_composited(
            &mut target,
            Point::zero(),
            embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK,
        )
        .unwrap();
    animation.advance();
    animation
        .draw_current_composited(
            &mut target,
            Point::zero(),
            embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK,
        )
        .unwrap();
    animation.advance();
    animation
        .draw_current_composited(
            &mut target,
            Point::zero(),
            embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK,
        )
        .unwrap();

    assert_eq!(
        target.pixels.as_slice(),
        &[
            Pixel(
                Point::new(0, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(1, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(2, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(0, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::RED
            ),
            Pixel(
                Point::new(2, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::GREEN
            ),
            Pixel(
                Point::new(0, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(1, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(2, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLACK
            ),
            Pixel(
                Point::new(0, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::RED
            ),
            Pixel(
                Point::new(1, 0),
                embedded_gif::embedded_graphics::pixelcolor::Rgb888::BLUE
            ),
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
