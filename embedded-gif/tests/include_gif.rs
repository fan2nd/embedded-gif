use embedded_gif::embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
};
use embedded_gif::{
    include_complete_gif, include_raw_gif, CompleteGif, CompleteGifFrame, DisposalMethod, RawGif,
    RawGifFrame, RawGifRleFrame, RawRleGif,
};

static COMPLETE_FRAMES: &[CompleteGifFrame] =
    include_complete_gif!("tests/fixtures/two_frames.gif");
static COMPLETE_RGB565_FRAMES: &[CompleteGifFrame<Rgb565>] =
    include_complete_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);
static COMPLETE_BINARY_FRAMES: &[CompleteGifFrame<BinaryColor>] = include_complete_gif!(
    "tests/fixtures/two_frames.gif",
    pixel_format = BinaryColor,
    dither = true
);
static RAW_FRAMES: &[RawGifFrame] = include_raw_gif!("tests/fixtures/two_frames.gif");
static RAW_RLE_FRAMES: &[RawGifRleFrame] =
    include_raw_gif!("tests/fixtures/two_frames.gif", compression = Rle);

#[test]
fn embeds_complete_frames_as_canvas_sized_image_raw_values() {
    assert_eq!(COMPLETE_FRAMES.len(), 2);
    assert_eq!(COMPLETE_FRAMES[0].image().size(), Size::new(1, 1));
    assert_eq!(COMPLETE_FRAMES[1].image().size(), Size::new(1, 1));
    assert_eq!(COMPLETE_FRAMES[0].alpha_mask(), &[0b1000_0000]);
    assert_eq!(COMPLETE_FRAMES[0].delay_centiseconds(), 10);
    assert_eq!(COMPLETE_FRAMES[1].delay_millis(), 100);
}

#[test]
fn embeds_complete_frames_in_selected_pixel_formats() {
    assert_eq!(COMPLETE_RGB565_FRAMES.len(), 2);
    assert_eq!(COMPLETE_RGB565_FRAMES[0].image().size(), Size::new(1, 1));

    assert_eq!(COMPLETE_BINARY_FRAMES.len(), 2);
    assert_eq!(COMPLETE_BINARY_FRAMES[0].image().size(), Size::new(1, 1));
}

#[test]
fn embeds_raw_frames_with_gif_semantics() {
    assert_eq!(RAW_FRAMES.len(), 2);
    assert_eq!(RAW_FRAMES[0].image().size(), Size::new(1, 1));
    assert_eq!(RAW_FRAMES[0].alpha_mask(), &[0b1000_0000]);
    assert_eq!(RAW_FRAMES[0].top_left(), Point::new(0, 0));
    assert_eq!(RAW_FRAMES[0].disposal_method(), DisposalMethod::Any);
}

#[test]
fn embeds_raw_frames_with_rle_compression() {
    assert_eq!(RAW_RLE_FRAMES.len(), 2);
    assert_eq!(RAW_RLE_FRAMES[0].size(), Size::new(1, 1));
    assert_eq!(RAW_RLE_FRAMES[0].runs().len(), 1);
    assert_eq!(RAW_RLE_FRAMES[0].runs()[0].skip(), 0);
    assert_eq!(RAW_RLE_FRAMES[0].runs()[0].len(), 1);
    assert_eq!(RAW_RLE_FRAMES[0].top_left(), Point::new(0, 0));
    assert_eq!(RAW_RLE_FRAMES[0].disposal_method(), DisposalMethod::Any);
}

#[test]
fn manages_complete_gif_timing() {
    let mut animation = CompleteGif::new(COMPLETE_FRAMES);

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
fn manages_raw_gif_timing() {
    let mut animation = RawGif::new(RAW_FRAMES);

    assert_eq!(animation.len(), 2);
    assert_eq!(animation.frame_index(), 0);
    assert!(animation.tick_centiseconds(10));
    assert_eq!(animation.frame_index(), 1);
}

#[test]
fn manages_raw_rle_gif_timing() {
    let mut animation = RawRleGif::new(RAW_RLE_FRAMES);

    assert_eq!(animation.len(), 2);
    assert_eq!(animation.frame_index(), 0);
    assert!(animation.tick_centiseconds(10));
    assert_eq!(animation.frame_index(), 1);
}
