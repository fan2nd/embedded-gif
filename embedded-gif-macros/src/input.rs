use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Ident, LitBool, LitStr, Token, TypePath,
};

pub(crate) struct IncludeGifInput {
    pub(crate) path: LitStr,
    pub(crate) options: IncludeGifOptions,
}

#[derive(Copy, Clone)]
pub(crate) struct IncludeGifOptions {
    pub(crate) pixel_format: PixelFormat,
    pub(crate) color: Option<PaletteColor>,
    pub(crate) dither: Dither,
}

impl Default for IncludeGifOptions {
    fn default() -> Self {
        Self {
            pixel_format: PixelFormat::Rgb888,
            color: None,
            dither: Dither::None,
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum PixelFormat {
    Rgb888,
    Rgb565,
    BinaryColor,
}

impl PixelFormat {
    pub(crate) fn palette_color(self) -> PaletteColor {
        match self {
            Self::Rgb888 => PaletteColor::Rgb888,
            Self::Rgb565 => PaletteColor::Rgb565,
            Self::BinaryColor => PaletteColor::BinaryColor,
        }
    }

    pub(crate) fn bits_per_pixel(self) -> u8 {
        match self {
            Self::Rgb888 => 24,
            Self::Rgb565 => 16,
            Self::BinaryColor => 1,
        }
    }

    pub(crate) fn frame_type(self, embedded_gif: &TokenStream2) -> TokenStream2 {
        let color = self.color_type(embedded_gif);
        quote!(#embedded_gif::GifFrame<#color>)
    }

    pub(crate) fn color_type(self, embedded_gif: &TokenStream2) -> TokenStream2 {
        match self {
            Self::Rgb888 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888),
            Self::Rgb565 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565),
            Self::BinaryColor => {
                quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum PaletteColor {
    Rgb888,
    Rgb565,
    BinaryColor,
}

impl PaletteColor {
    pub(crate) fn color_type(self, embedded_gif: &TokenStream2) -> TokenStream2 {
        match self {
            Self::Rgb888 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888),
            Self::Rgb565 => quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565),
            Self::BinaryColor => {
                quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor)
            }
        }
    }

    pub(crate) fn value(
        self,
        embedded_gif: &TokenStream2,
        red: u8,
        green: u8,
        blue: u8,
    ) -> TokenStream2 {
        match self {
            Self::Rgb888 => {
                quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb888::new(#red, #green, #blue))
            }
            Self::Rgb565 => {
                let red = red >> 3;
                let green = green >> 2;
                let blue = blue >> 3;
                quote!(#embedded_gif::embedded_graphics::pixelcolor::Rgb565::new(#red, #green, #blue))
            }
            Self::BinaryColor => {
                if (u16::from(red) * 299 + u16::from(green) * 587 + u16::from(blue) * 114) / 1000
                    >= 128
                {
                    quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor::On)
                } else {
                    quote!(#embedded_gif::embedded_graphics::pixelcolor::BinaryColor::Off)
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Dither {
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

            match key_ident.to_string().as_str() {
                "pixel_format" => options.pixel_format = parse_pixel_format(value)?,
                "color" => options.color = Some(parse_palette_color_option(value)?),
                "dither" => options.dither = parse_dither(value)?,
                _ => {
                    return Err(syn::Error::new(
                        key_ident.span(),
                        "supported options are `pixel_format`, `color`, and `dither`",
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
            "`pixel_format` expects a pixel color type",
        ));
    };
    let Some(segment) = value.path.segments.last() else {
        return Err(syn::Error::new(value.path.span(), "invalid pixel format"));
    };

    match segment.ident.to_string().as_str() {
        "Rgb888" => Ok(PixelFormat::Rgb888),
        "Rgb565" => Ok(PixelFormat::Rgb565),
        "BinaryColor" => Ok(PixelFormat::BinaryColor),
        _ => Err(syn::Error::new(
            segment.ident.span(),
            "supported pixel formats are `Rgb888`, `Rgb565`, and `BinaryColor`",
        )),
    }
}

fn parse_palette_color_option(value: OptionValue) -> syn::Result<PaletteColor> {
    let OptionValue::Type(value) = value else {
        return Err(syn::Error::new(
            Span::call_site(),
            "`color` expects `Rgb888`, `Rgb565`, or `BinaryColor`",
        ));
    };
    let Some(segment) = value.path.segments.last() else {
        return Err(syn::Error::new(value.path.span(), "invalid palette color"));
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
