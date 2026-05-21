use proc_macro::TokenStream;
use syn::parse_macro_input;

mod args;
mod attrs;
mod derive_type;
mod function;
mod module;
mod tokens;

#[cfg(test)]
mod tests;

#[proc_macro_derive(ZirconScriptType, attributes(zircon_script))]
pub fn derive_zircon_script_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_type::derive_zircon_script_type_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostFunctionArgs);
    let item = parse_macro_input!(item as syn::ItemFn);
    function::host_function_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostModuleArgs);
    let item = parse_macro_input!(item as syn::ItemMod);
    module::host_module_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
