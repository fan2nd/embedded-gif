use crate::input::{Dither, IncludeGifOptions, PixelFormat};

const TOKEN_RAW: u8 = 0b1000_0000;
const TOKEN_REPEAT: u8 = 0b1100_0000;

pub(crate) fn encode_frame(
    rgba: &[u8],
    width: u32,
    height: u32,
    options: IncludeGifOptions,
) -> Result<Vec<u8>, String> {
    if options.pixel_format.is_indexed() {
        return Err("indexed pixel formats require include_gif_indexed!".to_owned());
    }

    let binary_pixels = if matches!(options.pixel_format, PixelFormat::BinaryColor)
        && options.dither == Dither::FloydSteinberg
    {
        Some(BinaryDither::new(rgba, width, height)?)
    } else {
        None
    };
    let mut symbols = Vec::new();

    for (index, pixel) in rgba.chunks_exact(4).enumerate() {
        if pixel[3] == 0 {
            symbols.push(Symbol::Transparent);
            continue;
        }

        symbols.push(Symbol::Opaque(Color::from_rgba(
            options.pixel_format,
            pixel[0],
            pixel[1],
            pixel[2],
            binary_pixels.as_ref().map(|pixels| pixels.get(index)),
        )));
    }

    Ok(Encoder::new(options.pixel_format).encode(&symbols))
}

pub(crate) fn encode_indexed_frame(
    indices: &[u8],
    transparent: Option<u8>,
    pixel_format: PixelFormat,
) -> Vec<u8> {
    let symbols = indices
        .iter()
        .copied()
        .map(|index| {
            if Some(index) == transparent {
                Symbol::Transparent
            } else {
                Symbol::Opaque(Color::Index(index))
            }
        })
        .collect::<Vec<_>>();

    Encoder::new(pixel_format).encode(&symbols)
}

struct Encoder {
    pixel_format: PixelFormat,
    data: Vec<u8>,
}

impl Encoder {
    fn new(pixel_format: PixelFormat) -> Self {
        Self {
            pixel_format,
            data: Vec::new(),
        }
    }

    fn encode(mut self, symbols: &[Symbol]) -> Vec<u8> {
        let mut index = 0usize;

        while index < symbols.len() {
            match symbols[index] {
                Symbol::Transparent => {
                    let len = symbols[index..]
                        .iter()
                        .take_while(|symbol| matches!(symbol, Symbol::Transparent))
                        .count();
                    self.push_skip(len);
                    index += len;
                }
                Symbol::Opaque(color) => {
                    let repeat_len = symbols[index..]
                        .iter()
                        .take_while(
                            |symbol| matches!(symbol, Symbol::Opaque(next) if *next == color),
                        )
                        .count();
                    let opaque_len = symbols[index..]
                        .iter()
                        .take_while(|symbol| matches!(symbol, Symbol::Opaque(_)))
                        .count();

                    if repeat_len >= 2 || opaque_len == 1 {
                        self.push_repeat(color, repeat_len);
                        index += repeat_len;
                    } else {
                        let len = opaque_len.min(raw_until_repeat(&symbols[index..]));
                        self.push_raw(&symbols[index..index + len]);
                        index += len;
                    }
                }
            }
        }

        self.data
    }

    fn push_skip(&mut self, len: usize) {
        self.push_chunks(0, 128, len, |_| {});
    }

    fn push_repeat(&mut self, color: Color, len: usize) {
        let pixel_format = self.pixel_format;
        self.push_chunks(TOKEN_REPEAT, 64, len, |data| {
            let mut bits = Bits::default();
            color.push(data, &mut bits, pixel_format);
        });
    }

    fn push_raw(&mut self, symbols: &[Symbol]) {
        let mut offset = 0usize;

        while offset < symbols.len() {
            let chunk = (symbols.len() - offset).min(64);
            let mut bits = Bits::default();
            self.data.push(TOKEN_RAW | (chunk - 1) as u8);

            for symbol in &symbols[offset..offset + chunk] {
                let Symbol::Opaque(color) = *symbol else {
                    unreachable!("raw chunks can only contain opaque pixels");
                };
                color.push(&mut self.data, &mut bits, self.pixel_format);
            }

            offset += chunk;
        }
    }

    fn push_chunks(
        &mut self,
        token_kind: u8,
        max_len: usize,
        mut len: usize,
        mut payload: impl FnMut(&mut Vec<u8>),
    ) {
        while len > 0 {
            let chunk = len.min(max_len);
            self.data.push(token_kind | (chunk - 1) as u8);
            payload(&mut self.data);
            len -= chunk;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Symbol {
    Transparent,
    Opaque(Color),
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Color {
    Rgb(u8, u8, u8),
    Binary(bool),
    Index(u8),
}

impl Color {
    fn from_rgba(
        pixel_format: PixelFormat,
        red: u8,
        green: u8,
        blue: u8,
        binary: Option<bool>,
    ) -> Self {
        match pixel_format {
            PixelFormat::Rgb888 => Self::Rgb(red, green, blue),
            PixelFormat::Rgb565 => Self::Rgb(red >> 3, green >> 2, blue >> 3),
            PixelFormat::BinaryColor => {
                Self::Binary(binary.unwrap_or_else(|| luminance(red, green, blue) >= 128))
            }
            PixelFormat::PaletteIndex1(_)
            | PixelFormat::PaletteIndex2(_)
            | PixelFormat::PaletteIndex4(_)
            | PixelFormat::PaletteIndex8(_) => Self::Index(palette_index(
                luminance(red, green, blue),
                pixel_format.bits_per_pixel(),
            )),
        }
    }

    fn push(self, data: &mut Vec<u8>, bits: &mut Bits, pixel_format: PixelFormat) {
        match (pixel_format, self) {
            (PixelFormat::Rgb888, Self::Rgb(red, green, blue)) => {
                data.extend_from_slice(&[red, green, blue]);
            }
            (PixelFormat::Rgb565, Self::Rgb(red, green, blue)) => {
                let value = (u16::from(red) << 11) | (u16::from(green) << 5) | u16::from(blue);
                data.extend_from_slice(&value.to_be_bytes());
            }
            (PixelFormat::BinaryColor, Self::Binary(value)) => bits.push(data, u8::from(value), 1),
            (PixelFormat::PaletteIndex1(_), Self::Index(value))
            | (PixelFormat::PaletteIndex2(_), Self::Index(value))
            | (PixelFormat::PaletteIndex4(_), Self::Index(value))
            | (PixelFormat::PaletteIndex8(_), Self::Index(value)) => {
                bits.push(data, value, pixel_format.bits_per_pixel());
            }
            _ => unreachable!("pixel format and quantized color mismatch"),
        }
    }
}

#[derive(Default)]
struct Bits {
    offset: u8,
}

impl Bits {
    fn push(&mut self, data: &mut Vec<u8>, value: u8, len: u8) {
        for bit in (0..len).rev() {
            if self.offset == 0 {
                data.push(0);
            }

            let last = data.len() - 1;
            data[last] |= ((value >> bit) & 1) << (7 - self.offset);

            self.offset += 1;
            if self.offset == 8 {
                self.offset = 0;
            }
        }
    }
}

struct BinaryDither {
    pixels: Vec<bool>,
}

impl BinaryDither {
    fn new(rgba: &[u8], width: u32, height: u32) -> Result<Self, String> {
        let width = usize::try_from(width).map_err(|_| "GIF width is too large".to_owned())?;
        let height = usize::try_from(height).map_err(|_| "GIF height is too large".to_owned())?;
        let values = rgba
            .chunks_exact(4)
            .map(|rgba| luminance(rgba[0], rgba[1], rgba[2]))
            .collect::<Vec<_>>();

        Ok(Self {
            pixels: floyd_steinberg(values, width, height),
        })
    }

    fn get(&self, index: usize) -> bool {
        self.pixels[index]
    }
}

fn raw_until_repeat(symbols: &[Symbol]) -> usize {
    let mut len = 0usize;

    while len < symbols.len() {
        let Symbol::Opaque(color) = symbols[len] else {
            break;
        };

        let repeat_len = symbols[len..]
            .iter()
            .take_while(|symbol| matches!(symbol, Symbol::Opaque(next) if *next == color))
            .count();

        if len > 0 && repeat_len >= 2 {
            break;
        }

        len += 1;
    }

    len
}

fn palette_index(luminance: i16, bits_per_pixel: u8) -> u8 {
    let max = (1u16 << bits_per_pixel) - 1;
    ((luminance.clamp(0, 255) as u16 * max + 127) / 255) as u8
}

fn luminance(red: u8, green: u8, blue: u8) -> i16 {
    ((u16::from(red) * 299 + u16::from(green) * 587 + u16::from(blue) * 114) / 1000) as i16
}

fn floyd_steinberg(mut values: Vec<i16>, width: usize, height: usize) -> Vec<bool> {
    let mut pixels = vec![false; values.len()];

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let old = values[index].clamp(0, 255);
            let new = if old >= 128 { 255 } else { 0 };
            let error = old - new;
            pixels[index] = new == 255;

            diffuse(&mut values, width, height, x + 1, y, error, 7);

            if x > 0 {
                diffuse(&mut values, width, height, x - 1, y + 1, error, 3);
            }

            diffuse(&mut values, width, height, x, y + 1, error, 5);
            diffuse(&mut values, width, height, x + 1, y + 1, error, 1);
        }
    }

    pixels
}

fn diffuse(
    values: &mut [i16],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    error: i16,
    numerator: i16,
) {
    if x >= width || y >= height {
        return;
    }

    values[y * width + x] += error * numerator / 16;
}
