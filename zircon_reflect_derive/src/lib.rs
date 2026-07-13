use proc_macro::TokenStream;
use syn::parse_macro_input;

mod attributes;
mod derive;
mod fields;

#[cfg(test)]
mod tests;

#[proc_macro_derive(ZrReflect, attributes(zr_reflect))]
pub fn derive_zr_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive::derive_zr_reflect_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
