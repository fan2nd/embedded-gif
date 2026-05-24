use embedded_gif::embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
};
use embedded_gif::{include_gif, DisposalMethod, Gif, GifFrame};

static FRAMES: &[GifFrame] = include_gif!("tests/fixtures/two_frames.gif");
static RGB565_FRAMES: &[GifFrame<Rgb565>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);
static BINARY_FRAMES: &[GifFrame<BinaryColor>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = BinaryColor,
    dither = true
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
