use std::{env, fs::File, path::PathBuf};

use gif::{ColorOutput, DecodeOptions};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn include_gif(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr);

    match expand_include_gif(&path) {
        Ok(tokens) => tokens.into(),
        Err(message) => syn::Error::new(path.span(), message)
            .to_compile_error()
            .into(),
    }
}

fn expand_include_gif(path: &LitStr) -> Result<TokenStream2, String> {
    let gif_path = manifest_relative_path(path.value())?;
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
        let rgb = frame
            .buffer
            .chunks_exact(4)
            .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
            .collect::<Vec<_>>();
        let width = u32::from(frame.width);
        let delay = frame.delay;

        frame_tokens.push(quote! {
            {
                const #ident: &[u8] = &[#(#rgb),*];
                #embedded_gif::GifFrame::new(
                    #embedded_gif::embedded_graphics::image::ImageRaw::new(#ident, #width),
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

    Ok(quote! {
        &[
            #(#frame_tokens),*
        ] as &'static [#embedded_gif::GifFrame]
    })
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
