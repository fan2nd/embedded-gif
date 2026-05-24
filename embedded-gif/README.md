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

The macro supports pixel conversion and palette index output:

```rust
use embedded_gif::embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_gif::{include_gif, GifFrame, PaletteIndex4};

static RGB565: &[GifFrame<Rgb565>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);

static BINARY: &[GifFrame<BinaryColor>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = BinaryColor, dither = true);

static INDEXED: &[GifFrame<PaletteIndex4<Rgb565>>] =
    include_gif!("tests/fixtures/two_frames.gif", pixel_format = PaletteIndex4<Rgb565>);
```

Supported `pixel_format` values are `Rgb888`, `Rgb565`, `BinaryColor`,
`PaletteIndex1`, `PaletteIndex2`, `PaletteIndex4`, and `PaletteIndex8`.
Palette index formats quantize source pixels by luminance and pack indices at
1, 2, 4, or 8 bits per pixel inside the RLE byte stream. The palette index
types are generic over the palette entry color type, for example
`PaletteIndex4<Rgb565>`, and the color type is required.
`BinaryColor` can use Floyd-Steinberg dithering with `dither = true` or
`dither = FloydSteinberg`.

Macro paths are resolved relative to the calling crate's `CARGO_MANIFEST_DIR`.
