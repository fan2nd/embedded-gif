use std::{env, fs::File, path::PathBuf};

use gif::{ColorOutput, DecodeOptions, DisposalMethod as GifDisposalMethod};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    GenericArgument, Ident, LitBool, LitStr, PathArguments, Token, Type, TypePath,
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
    PaletteIndex1(PaletteColor),
    PaletteIndex2(PaletteColor),
    PaletteIndex4(PaletteColor),
    PaletteIndex8(PaletteColor),
}

#[derive(Copy, Clone)]
enum PaletteColor {
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
    Type(TypePath),
    Bool(LitBool),
}

impl Parse for OptionValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(LitBool) {
            Ok(Self::Bool(input.parse()?))
        } else {
            Ok(Self::Type(input.parse()?))
        }
    }
}

fn parse_pixel_format(value: OptionValue) -> syn::Result<PixelFormat> {
    let OptionValue::Type(value) = value else {
        return Err(syn::Error::new(
            Span::call_site(),
            "`pixel_format` expects `Rgb888`, `Rgb565`, `BinaryColor`, `PaletteIndex1`, `PaletteIndex2`, `PaletteIndex4`, or `PaletteIndex8`",
        ));
    };
    let Some(segment) = value.path.segments.last() else {
        return Err(syn::Error::new(value.path.span(), "invalid pixel format"));
    };
    match segment.ident.to_string().as_str() {
        "Rgb888" => Ok(PixelFormat::Rgb888),
        "Rgb565" => Ok(PixelFormat::Rgb565),
        "BinaryColor" => Ok(PixelFormat::BinaryColor),
        "PaletteIndex1" => Ok(PixelFormat::PaletteIndex1(parse_palette_color(
            &segment.arguments,
        )?)),
        "PaletteIndex2" => Ok(PixelFormat::PaletteIndex2(parse_palette_color(
            &segment.arguments,
        )?)),
        "PaletteIndex4" => Ok(PixelFormat::PaletteIndex4(parse_palette_color(
            &segment.arguments,
        )?)),
        "PaletteIndex8" => Ok(PixelFormat::PaletteIndex8(parse_palette_color(
            &segment.arguments,
        )?)),
        _ => Err(syn::Error::new(
            segment.ident.span(),
            "supported pixel formats are `Rgb888`, `Rgb565`, `BinaryColor`, `PaletteIndex1`, `PaletteIndex2`, `PaletteIndex4`, and `PaletteIndex8`",
        )),
    }
}

fn parse_palette_color(arguments: &PathArguments) -> syn::Result<PaletteColor> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return Err(syn::Error::new(
            Span::call_site(),
            "palette index formats require a color parameter, for example `PaletteIndex4<Rgb565>`",
        ));
    };

    let Some(GenericArgument::Type(Type::Path(color))) = arguments.args.first() else {
        return Err(syn::Error::new(
            arguments.span(),
            "palette index color parameter expects `Rgb888`, `Rgb565`, or `BinaryColor`",
        ));
    };
    let Some(segment) = color.path.segments.last() else {
        return Err(syn::Error::new(color.path.span(), "invalid palette color"));
    };

    match segment.ident.to_string().as_str() {
        "Rgb888" => Ok(PaletteColor::Rgb888),
        "Rgb565" => Ok(PaletteColor::Rgb565),
        "BinaryColor" => Ok(PaletteColor::BinaryColor),
        _ => Err(syn::Error::new(
            segment.ident.span(),
            "supported palette colors are `Rgb888`, `Rgb565`, and `BinaryColor`",
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
        OptionValue::Type(value) => match value
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .as_deref()
        {
            Some("None") => Ok(Dither::None),
            Some("FloydSteinberg") => Ok(Dither::FloydSteinberg),
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
    let mut reader = open_gif(&gif_path)?;
    let mut frame_tokens = Vec::new();
    let mut frame_count = 0usize;

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|error| format!("failed to read GIF frame: {error}"))?
    {
        let width = u32::from(frame.width);
        let height = u32::from(frame.height);
        let left = i32::from(frame.left);
        let top = i32::from(frame.top);
        let delay = frame.delay;
        let disposal = disposal_method(&embedded_gif, frame.dispose);
        let data_ident = format_ident!("__EMBEDDED_GIF_FRAME_{frame_count}");
        let data = rle_data(&frame.buffer, width, height, input.options)?;

        frame_tokens.push(quote! {
            {
                const #data_ident: &[u8] = &[#(#data),*];
                #embedded_gif::GifFrame::new(
                    #data_ident,
                    #embedded_gif::embedded_graphics::geometry::Size::new(#width, #height),
                    #embedded_gif::embedded_graphics::geometry::Point::new(#left, #top),
                    #delay,
                    #disposal,
                )
            }
        });

        frame_count += 1;
    }

    ensure_frames(&gif_path, frame_count)?;

    let frame_type = frame_type(&embedded_gif, input.options.pixel_format);
    Ok(quote! {
        &[
            #(#frame_tokens),*
        ] as &'static [#frame_type]
    })
}

fn open_gif(gif_path: &PathBuf) -> Result<gif::Decoder<File>, String> {
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::RGBA);

    let file = File::open(gif_path)
        .map_err(|error| format!("failed to open GIF '{}': {error}", gif_path.display()))?;
    options
        .read_info(file)
        .map_err(|error| format!("failed to decode GIF '{}': {error}", gif_path.display()))
}

fn ensure_frames(gif_path: &PathBuf, frame_count: usize) -> Result<(), String> {
    if frame_count == 0 {
        return Err(format!(
            "GIF '{}' does not contain any frames",
            gif_path.display()
        ));
    }

    Ok(())
}

fn color_type(embedded_gif: &TokenStream2, pixel_format: PixelFormat) -> TokenStream2 {
    match pixel_format {
        PixelFormat::Rgb888 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888),
        PixelFormat::Rgb565 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565),
        PixelFormat::BinaryColor => {
            quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor)
        }
        PixelFormat::PaletteIndex1(color) => {
            let color = palette_color_type(embedded_gif, color);
            quote!(#embedded_gif::PaletteIndex1<#color>)
        }
        PixelFormat::PaletteIndex2(color) => {
            let color = palette_color_type(embedded_gif, color);
            quote!(#embedded_gif::PaletteIndex2<#color>)
        }
        PixelFormat::PaletteIndex4(color) => {
            let color = palette_color_type(embedded_gif, color);
            quote!(#embedded_gif::PaletteIndex4<#color>)
        }
        PixelFormat::PaletteIndex8(color) => {
            let color = palette_color_type(embedded_gif, color);
            quote!(#embedded_gif::PaletteIndex8<#color>)
        }
    }
}

fn palette_color_type(embedded_gif: &TokenStream2, color: PaletteColor) -> TokenStream2 {
    match color {
        PaletteColor::Rgb888 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888),
        PaletteColor::Rgb565 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565),
        PaletteColor::BinaryColor => {
            quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor)
        }
    }
}

fn frame_type(embedded_gif: &TokenStream2, pixel_format: PixelFormat) -> TokenStream2 {
    let color = color_type(embedded_gif, pixel_format);
    quote!(#embedded_gif::GifFrame<#color>)
}

fn disposal_method(
    embedded_gif: &TokenStream2,
    disposal_method: GifDisposalMethod,
) -> TokenStream2 {
    match disposal_method {
        GifDisposalMethod::Any => quote!(#embedded_gif::DisposalMethod::Any),
        GifDisposalMethod::Keep => quote!(#embedded_gif::DisposalMethod::Keep),
        GifDisposalMethod::Background => quote!(#embedded_gif::DisposalMethod::Background),
        GifDisposalMethod::Previous => quote!(#embedded_gif::DisposalMethod::Previous),
    }
}

fn rle_data(
    rgba: &[u8],
    width: u32,
    height: u32,
    options: IncludeGifOptions,
) -> Result<Vec<u8>, String> {
    let dithered_binary = if matches!(options.pixel_format, PixelFormat::BinaryColor)
        && options.dither == Dither::FloydSteinberg
    {
        Some(dithered_binary_pixels(rgba, width, height)?)
    } else {
        None
    };
    let mut symbols = Vec::new();

    for (index, pixel) in rgba.chunks_exact(4).enumerate() {
        if pixel[3] == 0 {
            symbols.push(RleSymbol::Transparent);
            continue;
        }

        symbols.push(RleSymbol::Opaque(quantized_color(
            options.pixel_format,
            pixel[0],
            pixel[1],
            pixel[2],
            dithered_binary.as_ref().map(|pixels| pixels[index]),
        )));
    }

    Ok(encode_symbols(&symbols, options.pixel_format))
}

fn encode_symbols(symbols: &[RleSymbol], pixel_format: PixelFormat) -> Vec<u8> {
    let mut data = Vec::new();
    let mut index = 0usize;

    while index < symbols.len() {
        match symbols[index] {
            RleSymbol::Transparent => {
                let len = matching_len(symbols, index, |symbol| {
                    matches!(symbol, RleSymbol::Transparent)
                });
                push_skip(&mut data, len);
                index += len;
            }
            RleSymbol::Opaque(color) => {
                let solid_len = matching_len(
                    symbols,
                    index,
                    |symbol| matches!(symbol, RleSymbol::Opaque(next) if next == color),
                );
                let raw_len = opaque_len(symbols, index);

                if solid_len >= 2 || raw_len == 1 {
                    push_solid(&mut data, pixel_format, color, solid_len);
                    index += solid_len;
                } else {
                    let len = raw_len.min(raw_until_solid(symbols, index));
                    push_raw(&mut data, pixel_format, &symbols[index..index + len]);
                    index += len;
                }
            }
        }
    }

    data
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum RleSymbol {
    Transparent,
    Opaque(QuantizedColor),
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum QuantizedColor {
    Rgb(u8, u8, u8),
    Binary(bool),
    Index(u8),
}

fn matching_len(
    symbols: &[RleSymbol],
    start: usize,
    mut matches: impl FnMut(RleSymbol) -> bool,
) -> usize {
    symbols[start..]
        .iter()
        .copied()
        .take_while(|symbol| matches(*symbol))
        .count()
}

fn opaque_len(symbols: &[RleSymbol], start: usize) -> usize {
    matching_len(symbols, start, |symbol| {
        matches!(symbol, RleSymbol::Opaque(_))
    })
}

fn raw_until_solid(symbols: &[RleSymbol], start: usize) -> usize {
    let mut len = 0usize;

    while start + len < symbols.len() {
        let RleSymbol::Opaque(color) = symbols[start + len] else {
            break;
        };

        let solid_len = matching_len(
            symbols,
            start + len,
            |symbol| matches!(symbol, RleSymbol::Opaque(next) if next == color),
        );

        if len > 0 && solid_len >= 2 {
            break;
        }

        len += 1;
    }

    len
}

fn push_skip(data: &mut Vec<u8>, mut len: usize) {
    while len > 0 {
        let chunk = len.min(64);
        data.push((chunk - 1) as u8);
        len -= chunk;
    }
}

fn push_solid(
    data: &mut Vec<u8>,
    pixel_format: PixelFormat,
    color: QuantizedColor,
    mut len: usize,
) {
    while len > 0 {
        let chunk = len.min(64);
        data.push(0b0100_0000 | (chunk - 1) as u8);
        push_color(data, pixel_format, color, &mut 0);
        len -= chunk;
    }
}

fn push_raw(data: &mut Vec<u8>, pixel_format: PixelFormat, symbols: &[RleSymbol]) {
    let mut offset = 0usize;

    while offset < symbols.len() {
        let chunk = (symbols.len() - offset).min(64);
        let mut bit_offset = 0;
        data.push(0b1000_0000 | (chunk - 1) as u8);

        for symbol in &symbols[offset..offset + chunk] {
            let RleSymbol::Opaque(color) = *symbol else {
                unreachable!("raw chunks can only contain opaque pixels");
            };
            push_color(data, pixel_format, color, &mut bit_offset);
        }

        offset += chunk;
    }
}

fn push_color(
    data: &mut Vec<u8>,
    pixel_format: PixelFormat,
    color: QuantizedColor,
    bit_offset: &mut u8,
) {
    match (pixel_format, color) {
        (PixelFormat::Rgb888, QuantizedColor::Rgb(red, green, blue)) => {
            data.extend_from_slice(&[red, green, blue]);
        }
        (PixelFormat::Rgb565, QuantizedColor::Rgb(red, green, blue)) => {
            let value = (u16::from(red) << 11) | (u16::from(green) << 5) | u16::from(blue);
            data.extend_from_slice(&value.to_be_bytes());
        }
        (PixelFormat::BinaryColor, QuantizedColor::Binary(value)) => data.push(u8::from(value)),
        (PixelFormat::PaletteIndex1(_), QuantizedColor::Index(value))
        | (PixelFormat::PaletteIndex2(_), QuantizedColor::Index(value))
        | (PixelFormat::PaletteIndex4(_), QuantizedColor::Index(value))
        | (PixelFormat::PaletteIndex8(_), QuantizedColor::Index(value)) => {
            push_bits(data, bit_offset, value, pixel_format.bits_per_pixel())
        }
        _ => unreachable!("pixel format and quantized color mismatch"),
    }
}

fn dithered_binary_pixels(rgba: &[u8], width: u32, height: u32) -> Result<Vec<bool>, String> {
    let width = usize::try_from(width).map_err(|_| "GIF width is too large".to_owned())?;
    let height = usize::try_from(height).map_err(|_| "GIF height is too large".to_owned())?;
    let luminance = rgba
        .chunks_exact(4)
        .map(|rgba| grayscale(rgba[0], rgba[1], rgba[2]))
        .collect::<Vec<_>>();

    Ok(floyd_steinberg(luminance, width, height))
}

fn quantized_color(
    pixel_format: PixelFormat,
    red: u8,
    green: u8,
    blue: u8,
    dithered_binary: Option<bool>,
) -> QuantizedColor {
    match pixel_format {
        PixelFormat::Rgb888 => QuantizedColor::Rgb(red, green, blue),
        PixelFormat::Rgb565 => QuantizedColor::Rgb(red >> 3, green >> 2, blue >> 3),
        PixelFormat::BinaryColor => QuantizedColor::Binary(
            dithered_binary.unwrap_or_else(|| grayscale(red, green, blue) >= 128),
        ),
        PixelFormat::PaletteIndex1(_)
        | PixelFormat::PaletteIndex2(_)
        | PixelFormat::PaletteIndex4(_)
        | PixelFormat::PaletteIndex8(_) => QuantizedColor::Index(palette_index(
            grayscale(red, green, blue),
            pixel_format.bits_per_pixel(),
        )),
    }
}

impl PixelFormat {
    fn bits_per_pixel(self) -> u8 {
        match self {
            PixelFormat::Rgb888 => 24,
            PixelFormat::Rgb565 => 16,
            PixelFormat::BinaryColor | PixelFormat::PaletteIndex1(_) => 1,
            PixelFormat::PaletteIndex2(_) => 2,
            PixelFormat::PaletteIndex4(_) => 4,
            PixelFormat::PaletteIndex8(_) => 8,
        }
    }
}

fn palette_index(luminance: i16, bits_per_pixel: u8) -> u8 {
    let max = (1u16 << bits_per_pixel) - 1;
    ((luminance.clamp(0, 255) as u16 * max + 127) / 255) as u8
}

fn push_bits(data: &mut Vec<u8>, bit_offset: &mut u8, value: u8, bits_per_pixel: u8) {
    for bit in (0..bits_per_pixel).rev() {
        if *bit_offset == 0 {
            data.push(0);
        }

        let last = data.len() - 1;
        data[last] |= ((value >> bit) & 1) << (7 - *bit_offset);

        *bit_offset += 1;
        if *bit_offset == 8 {
            *bit_offset = 0;
        }
    }
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
