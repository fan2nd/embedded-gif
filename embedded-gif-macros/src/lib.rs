use std::{env, fs::File, path::PathBuf};

use gif::{ColorOutput, DecodeOptions};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitBool, LitStr, Token,
};

#[proc_macro]
pub fn include_gif(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IncludeGifInput);

    match expand_include_gif(&input) {
        Ok(tokens) => tokens.into(),
        Err(message) => syn::Error::new(input.path.span(), message)
            .to_compile_error()
            .into(),
    }
}

struct IncludeGifInput {
    path: LitStr,
    options: IncludeGifOptions,
}

#[derive(Copy, Clone)]
struct IncludeGifOptions {
    pixel_format: PixelFormat,
    dither: Dither,
}

impl Default for IncludeGifOptions {
    fn default() -> Self {
        Self {
            pixel_format: PixelFormat::Rgb888,
            dither: Dither::None,
        }
    }
}

#[derive(Copy, Clone)]
enum PixelFormat {
    Rgb888,
    Rgb565,
    BinaryColor,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Dither {
    None,
    FloydSteinberg,
}

impl Parse for IncludeGifInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = input.parse()?;
        let mut options = IncludeGifOptions::default();

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.is_empty() {
                break;
            }

            let key_ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: OptionValue = input.parse()?;
            let key = key_ident.to_string();

            match key.as_str() {
                "pixel_format" => options.pixel_format = parse_pixel_format(value)?,
                "dither" => options.dither = parse_dither(value)?,
                _ => {
                    return Err(syn::Error::new(
                        key_ident.span(),
                        "supported options are `pixel_format` and `dither`",
                    ));
                }
            }
        }

        Ok(Self { path, options })
    }
}

enum OptionValue {
    Ident(Ident),
    Bool(LitBool),
}

impl Parse for OptionValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(LitBool) {
            Ok(Self::Bool(input.parse()?))
        } else {
            Ok(Self::Ident(input.parse()?))
        }
    }
}

fn parse_pixel_format(value: OptionValue) -> syn::Result<PixelFormat> {
    let OptionValue::Ident(value) = value else {
        return Err(syn::Error::new(
            Span::call_site(),
            "`pixel_format` expects `Rgb888`, `Rgb565`, or `BinaryColor`",
        ));
    };

    match value.to_string().as_str() {
        "Rgb888" => Ok(PixelFormat::Rgb888),
        "Rgb565" => Ok(PixelFormat::Rgb565),
        "BinaryColor" => Ok(PixelFormat::BinaryColor),
        _ => Err(syn::Error::new(
            value.span(),
            "supported pixel formats are `Rgb888`, `Rgb565`, and `BinaryColor`",
        )),
    }
}

fn parse_dither(value: OptionValue) -> syn::Result<Dither> {
    match value {
        OptionValue::Bool(value) => {
            if value.value {
                Ok(Dither::FloydSteinberg)
            } else {
                Ok(Dither::None)
            }
        }
        OptionValue::Ident(value) => match value.to_string().as_str() {
            "None" => Ok(Dither::None),
            "FloydSteinberg" => Ok(Dither::FloydSteinberg),
            _ => Err(syn::Error::new(
                value.span(),
                "supported dither values are `true`, `false`, `None`, and `FloydSteinberg`",
            )),
        },
    }
}

fn expand_include_gif(input: &IncludeGifInput) -> Result<TokenStream2, String> {
    if input.options.dither != Dither::None
        && !matches!(input.options.pixel_format, PixelFormat::BinaryColor)
    {
        return Err("`dither` is only supported with `pixel_format = BinaryColor`".to_owned());
    }

    let gif_path = manifest_relative_path(input.path.value())?;
    let embedded_gif = crate_path()?;
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::RGBA);

    let file = File::open(&gif_path)
        .map_err(|error| format!("failed to open GIF '{}': {error}", gif_path.display()))?;
    let mut reader = options
        .read_info(file)
        .map_err(|error| format!("failed to decode GIF '{}': {error}", gif_path.display()))?;

    let mut frame_tokens = Vec::new();
    let mut frame_count = 0usize;

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|error| format!("failed to read GIF frame: {error}"))?
    {
        let ident = format_ident!("__EMBEDDED_GIF_FRAME_{frame_count}");
        let width = u32::from(frame.width);
        let height = u32::from(frame.height);
        let delay = frame.delay;
        let bytes = convert_frame(&frame.buffer, width, height, input.options)?;
        let color = color_type(&embedded_gif, input.options.pixel_format);

        frame_tokens.push(quote! {
            {
                const #ident: &[u8] = &[#(#bytes),*];
                #embedded_gif::GifFrame::new(
                    #embedded_gif::embedded_graphics::image::ImageRaw::<#color>::new(#ident, #width),
                    #delay,
                )
            }
        });
        frame_count += 1;
    }

    if frame_count == 0 {
        return Err(format!(
            "GIF '{}' does not contain any frames",
            gif_path.display()
        ));
    }

    let frame_type = frame_type(&embedded_gif, input.options.pixel_format);

    Ok(quote! {
        &[
            #(#frame_tokens),*
        ] as &'static [#frame_type]
    })
}

fn color_type(embedded_gif: &TokenStream2, pixel_format: PixelFormat) -> TokenStream2 {
    match pixel_format {
        PixelFormat::Rgb888 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888),
        PixelFormat::Rgb565 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565),
        PixelFormat::BinaryColor => {
            quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor)
        }
    }
}

fn frame_type(embedded_gif: &TokenStream2, pixel_format: PixelFormat) -> TokenStream2 {
    let color = color_type(embedded_gif, pixel_format);
    quote!(#embedded_gif::GifFrame<#color>)
}

fn convert_frame(
    rgba: &[u8],
    width: u32,
    height: u32,
    options: IncludeGifOptions,
) -> Result<Vec<u8>, String> {
    match options.pixel_format {
        PixelFormat::Rgb888 => Ok(rgba
            .chunks_exact(4)
            .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
            .collect()),
        PixelFormat::Rgb565 => Ok(rgba
            .chunks_exact(4)
            .flat_map(|rgba| rgb565_be(rgba[0], rgba[1], rgba[2]))
            .collect()),
        PixelFormat::BinaryColor => binary_frame(rgba, width, height, options.dither),
    }
}

fn rgb565_be(red: u8, green: u8, blue: u8) -> [u8; 2] {
    let value =
        ((u16::from(red) & 0xF8) << 8) | ((u16::from(green) & 0xFC) << 3) | (u16::from(blue) >> 3);
    value.to_be_bytes()
}

fn binary_frame(rgba: &[u8], width: u32, height: u32, dither: Dither) -> Result<Vec<u8>, String> {
    let width = usize::try_from(width).map_err(|_| "GIF width is too large".to_owned())?;
    let height = usize::try_from(height).map_err(|_| "GIF height is too large".to_owned())?;
    let luminance = rgba
        .chunks_exact(4)
        .map(|rgba| grayscale(rgba[0], rgba[1], rgba[2]))
        .collect::<Vec<_>>();

    let pixels = match dither {
        Dither::None => luminance
            .into_iter()
            .map(|value| value >= 128)
            .collect::<Vec<_>>(),
        Dither::FloydSteinberg => floyd_steinberg(luminance, width, height),
    };

    Ok(pack_binary_rows(&pixels, width, height))
}

fn grayscale(red: u8, green: u8, blue: u8) -> i16 {
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

    let index = y * width + x;
    values[index] += error * numerator / 16;
}

fn pack_binary_rows(pixels: &[bool], width: usize, height: usize) -> Vec<u8> {
    let bytes_per_row = (width + 7) / 8;
    let mut data = vec![0; bytes_per_row * height];

    for y in 0..height {
        for x in 0..width {
            if pixels[y * width + x] {
                data[y * bytes_per_row + x / 8] |= 0x80 >> (x % 8);
            }
        }
    }

    data
}

fn manifest_relative_path(path: String) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    if path.is_absolute() {
        return Ok(path);
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR is not set while expanding include_gif!".to_owned())?;

    Ok(PathBuf::from(manifest_dir).join(path))
}

fn crate_path() -> Result<TokenStream2, String> {
    match crate_name("embedded-gif").map_err(|error| error.to_string())? {
        FoundCrate::Itself => Ok(quote!(::embedded_gif)),
        FoundCrate::Name(name) => {
            let ident = proc_macro2::Ident::new(&name, Span::call_site());
            Ok(quote!(::#ident))
        }
    }
}
