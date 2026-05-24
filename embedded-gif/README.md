# embedded-gif

Embed GIF frames as static `embedded-graphics` `ImageRaw<Rgb888>` values.

```rust
use embedded_gif::{include_gif, GifAnimation};

static FRAMES: &[embedded_gif::GifFrame] = include_gif!("tests/fixtures/two_frames.gif");

let mut animation = GifAnimation::new(FRAMES);
animation.advance_by_millis(100);
```

The `include_gif!` path is resolved relative to the calling crate's
`CARGO_MANIFEST_DIR`. Each GIF frame is decoded to RGB888 data and returned with
its GIF delay metadata. `GifAnimation` keeps the current frame index, advances by
elapsed time, and can draw the current frame to an `embedded-graphics` draw
target.
