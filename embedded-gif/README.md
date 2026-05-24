# embedded-gif

Embed GIF assets for `embedded-graphics`.

The public API is intentionally small:

```rust
use embedded_gif::{include_gif_frames, Gif, GifFrame};

static FRAMES: &[GifFrame] = include_gif_frames!("tests/fixtures/two_frames.gif");

let mut animation = Gif::new(FRAMES);
animation.tick_millis(100);
```

`include_gif_frames!` preserves raw GIF frame semantics. Each generated `GifFrame`
stores its frame-local size, top-left offset, delay, disposal method, and a
mandatory compact RLE byte stream. Transparent pixels are encoded as skip tokens,
so delta drawing keeps GIF transparency semantics without a separate alpha mask.
RLE tokens use `0xxxxxxx` for transparent skips, `10xxxxxx` for raw pixels, and
`11xxxxxx` for repeated pixels.

Use `draw_current_delta` when the caller owns the framebuffer and wants to draw
only the current frame delta. Use `draw_current_composited` when the caller wants
the library to replay raw frames and draw the complete current visual state.

`include_gif_frames!` supports direct pixel conversion:

```rust
use embedded_gif::embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_gif::{include_gif_frames, GifFrame};

static RGB565: &[GifFrame<Rgb565>] =
    include_gif_frames!("tests/fixtures/two_frames.gif", pixel_format = Rgb565);

static BINARY: &[GifFrame<BinaryColor>] =
    include_gif_frames!("tests/fixtures/two_frames.gif", pixel_format = BinaryColor, dither = true);
```

Use `include_gif_indexed!` to preserve indexed GIF data and embed the source
palette:

```rust
use embedded_gif::embedded_graphics::pixelcolor::Rgb565;
use embedded_gif::{include_gif_indexed, IndexedGif, PaletteIndex4};

static INDEXED: IndexedGif<PaletteIndex4<Rgb565>, Rgb565> =
    include_gif_indexed!("tests/fixtures/two_frames.gif", pixel_format = PaletteIndex4<Rgb565>);
```

Direct `pixel_format` values are `Rgb888`, `Rgb565`, and `BinaryColor`.
`BinaryColor` can use Floyd-Steinberg dithering with `dither = true` or
`dither = FloydSteinberg`. Indexed `pixel_format` values are `PaletteIndex1`,
`PaletteIndex2`, `PaletteIndex4`, and `PaletteIndex8`; they require
`include_gif_indexed!` so the palette is included in the generated value.

Macro paths are resolved relative to the calling crate's `CARGO_MANIFEST_DIR`.
