# embedded-gif

Embed GIF assets for `embedded-graphics`.

There are two explicit modes:

```rust
use embedded_gif::{include_complete_gif, include_raw_gif, CompleteGif, CompleteGifFrame, RawGifFrame};

static COMPLETE: &[CompleteGifFrame] = include_complete_gif!("tests/fixtures/two_frames.gif");
static RAW: &[RawGifFrame] = include_raw_gif!("tests/fixtures/two_frames.gif");

let mut animation = CompleteGif::new(COMPLETE);
animation.tick_millis(100);
```

`include_complete_gif!` composites GIF frames onto the logical canvas at compile
time. Each generated frame is a full canvas-sized `ImageRaw` plus a 1bpp alpha
mask, so transparent canvas pixels are skipped when drawing.

`include_raw_gif!` preserves GIF frame semantics. Each generated frame keeps its
own image, 1bpp alpha mask, top-left offset, delay, and disposal method. Use
`draw_current_delta` when the caller maintains the framebuffer and applies only
the current diff. Use `draw_current_composited` when the caller wants raw storage
but wants the library to replay raw frames and draw the complete current visual
state.

Raw GIFs can also use RLE compression:

```rust
use embedded_gif::{include_raw_gif, RawGifRleFrame, RawRleGif};

static RAW_RLE: &[RawGifRleFrame] =
    include_raw_gif!("tests/fixtures/two_frames.gif", compression = Rle);

let mut animation = RawRleGif::new(RAW_RLE);
animation.tick_millis(100);
```

RLE stores transparent gaps and same-color opaque runs instead of per-pixel image
data plus an alpha mask.

Both macros support pixel conversion:

```rust
use embedded_gif::embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_gif::{include_complete_gif, CompleteGifFrame};

static RGB565: &[CompleteGifFrame<Rgb565>] =
    include_complete_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);

static BINARY: &[CompleteGifFrame<BinaryColor>] =
    include_complete_gif!("tests/fixtures/two_frames.gif", pixel_format = BinaryColor, dither = true);
```

Supported `pixel_format` values are `Rgb888`, `Rgb565`, and `BinaryColor`.
`BinaryColor` can use Floyd-Steinberg dithering with `dither = true` or
`dither = FloydSteinberg`.

Macro paths are resolved relative to the calling crate's `CARGO_MANIFEST_DIR`.
