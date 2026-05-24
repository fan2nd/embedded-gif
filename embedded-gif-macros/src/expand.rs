use std::{env, fs::File, path::PathBuf};

use gif::{ColorOutput, DecodeOptions, DisposalMethod as GifDisposalMethod};
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};

use crate::{
    input::{Dither, IncludeGifInput, PixelFormat},
    rle,
};

pub(crate) fn include_gif_frames(input: &IncludeGifInput) -> Result<TokenStream2, String> {
    if input.options.pixel_format.is_indexed() {
        return Err(
            "`PaletteIndex*` output requires `include_gif_indexed!` so the palette is included"
                .to_owned(),
        );
    }

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
        let data = rle::encode_frame(&frame.buffer, width, height, input.options)?;

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

    if frame_count == 0 {
        return Err(format!(
            "GIF '{}' does not contain any frames",
            gif_path.display()
        ));
    }

    let frame_type = input.options.pixel_format.frame_type(&embedded_gif);
    Ok(quote! {
        &[
            #(#frame_tokens),*
        ] as &'static [#frame_type]
    })
}

pub(crate) fn include_gif_indexed(input: &IncludeGifInput) -> Result<TokenStream2, String> {
    if !input.options.pixel_format.is_indexed() {
        return Err(
            "`include_gif_indexed!` requires `pixel_format = PaletteIndexN<Color>`".to_owned(),
        );
    }
    if input.options.dither != Dither::None {
        return Err("`dither` is not supported by indexed GIF output".to_owned());
    }

    let gif_path = manifest_relative_path(input.path.value())?;
    let embedded_gif = crate_path()?;
    let mut reader = open_indexed_gif(&gif_path)?;
    let palette = reader
        .global_palette()
        .ok_or_else(|| {
            format!(
                "GIF '{}' does not contain a global palette for indexed output",
                gif_path.display()
            )
        })?
        .to_vec();
    let mut frame_tokens = Vec::new();
    let mut frame_count = 0usize;

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|error| format!("failed to read GIF frame: {error}"))?
    {
        if frame.palette.is_some() {
            return Err("indexed GIF output currently requires a single global palette".to_owned());
        }

        let width = u32::from(frame.width);
        let height = u32::from(frame.height);
        let left = i32::from(frame.left);
        let top = i32::from(frame.top);
        let delay = frame.delay;
        let disposal = disposal_method(&embedded_gif, frame.dispose);
        let data_ident = format_ident!("__EMBEDDED_GIF_INDEXED_FRAME_{frame_count}");
        let data =
            rle::encode_indexed_frame(&frame.buffer, frame.transparent, input.options.pixel_format);

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

    if frame_count == 0 {
        return Err(format!(
            "GIF '{}' does not contain any frames",
            gif_path.display()
        ));
    }

    let palette_color = input
        .options
        .pixel_format
        .palette_color()
        .ok_or_else(|| "indexed output requires a palette index pixel format".to_owned())?;
    let palette_type = palette_color.color_type(&embedded_gif);
    let palette_entries = palette
        .chunks_exact(3)
        .map(|rgb| {
            let red = rgb[0];
            let green = rgb[1];
            let blue = rgb[2];
            palette_color.value(&embedded_gif, red, green, blue)
        })
        .collect::<Vec<_>>();
    let index_type = input.options.pixel_format.color_type(&embedded_gif);
    let frame_type = input.options.pixel_format.frame_type(&embedded_gif);

    Ok(quote! {
        {
            const PALETTE: &[#palette_type] = &[#(#palette_entries),*];
            const FRAMES: &[#frame_type] = &[
                #(#frame_tokens),*
            ];

            #embedded_gif::IndexedGif::<#index_type, #palette_type>::new(FRAMES, PALETTE)
        }
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

fn open_indexed_gif(gif_path: &PathBuf) -> Result<gif::Decoder<File>, String> {
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::Indexed);

    let file = File::open(gif_path)
        .map_err(|error| format!("failed to open GIF '{}': {error}", gif_path.display()))?;
    options
        .read_info(file)
        .map_err(|error| format!("failed to decode GIF '{}': {error}", gif_path.display()))
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

fn manifest_relative_path(path: String) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    if path.is_absolute() {
        return Ok(path);
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        "CARGO_MANIFEST_DIR is not set while expanding include_gif_frames!".to_owned()
    })?;

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
