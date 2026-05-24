use embedded_gif::embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
};
use embedded_gif::{include_gif, GifAnimation, GifFrame};

static FRAMES: &[GifFrame] = include_gif!("tests/fixtures/two_frames.gif");
static RGB565_FRAMES: &[GifFrame<Rgb565>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);
static BINARY_FRAMES: &[GifFrame<BinaryColor>] = include_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = BinaryColor,
    dither = true
);

#[test]
fn embeds_each_gif_frame_as_rgb888_image_raw() {
    assert_eq!(FRAMES.len(), 2);
    assert_eq!(FRAMES[0].image().size(), Size::new(1, 1));
    assert_eq!(FRAMES[1].image().size(), Size::new(1, 1));
    assert_eq!(FRAMES[0].delay_centiseconds(), 10);
    assert_eq!(FRAMES[1].delay_millis(), 100);
}

#[test]
fn embeds_selected_pixel_formats() {
    assert_eq!(RGB565_FRAMES.len(), 2);
    assert_eq!(RGB565_FRAMES[0].image().size(), Size::new(1, 1));

    assert_eq!(BINARY_FRAMES.len(), 2);
    assert_eq!(BINARY_FRAMES[0].image().size(), Size::new(1, 1));
}

#[test]
fn manages_current_frame_from_gif_delays() {
    let mut animation = GifAnimation::new(FRAMES);

    assert_eq!(animation.len(), 2);
    assert_eq!(animation.frame_index(), 0);
    assert!(!animation.advance_by_millis(95));
    assert_eq!(animation.frame_index(), 0);
    assert!(animation.advance_by_millis(5));
    assert_eq!(animation.frame_index(), 1);
    assert!(animation.advance_by_centiseconds(10));
    assert_eq!(animation.frame_index(), 0);
}
