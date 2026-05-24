# embedded-gif

Embed GIF assets for `embedded-graphics`.

The public API is intentionally small:

```rust
use embedded_gif::{include_gif, Gif, GifFrame};

static FRAMES: &[GifFrame] = include_gif!("tests/fixtures/two_frames.gif");

let mut animation = Gif::new(FRAMES);
animation.tick_millis(100);
```

`include_gif!` preserves raw GIF frame semantics. Each generated `GifFrame`
stores its frame-local size, top-left offset, delay, disposal method, and a
mandatory compact RLE byte stream. Transparent pixels are encoded as skip tokens,
so delta drawing keeps GIF transparency semantics without a separate alpha mask.

Use `draw_current_delta` when the caller owns the framebuffer and wants to draw
only the current frame delta. Use `draw_current_composited` when the caller wants
the library to replay raw frames and draw the complete current visual state.

The macro supports pixel conversion:

```rust
use embedded_gif::embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_gif::{include_gif, GifFrame};

static RGB565: &[GifFrame<Rgb565>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);

static BINARY: &[GifFrame<BinaryColor>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = BinaryColor, dither = true);
```

Supported `pixel_format` values are `Rgb888`, `Rgb565`, and `BinaryColor`.
`BinaryColor` can use Floyd-Steinberg dithering with `dither = true` or
`dither = FloydSteinberg`.

Macro paths are resolved relative to the calling crate's `CARGO_MANIFEST_DIR`.
