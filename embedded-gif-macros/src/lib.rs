mod expand;
mod input;
mod rle;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn include_gif_frames(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as input::IncludeGifInput);

    match expand::include_gif_frames(&input) {
        Ok(tokens) => tokens.into(),
        Err(message) => syn::Error::new(input.path.span(), message)
            .to_compile_error()
            .into(),
    }
}

#[proc_macro]
pub fn include_gif_indexed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as input::IncludeGifInput);

    match expand::include_gif_indexed(&input) {
        Ok(tokens) => tokens.into(),
        Err(message) => syn::Error::new(input.path.span(), message)
            .to_compile_error()
            .into(),
    }
}
