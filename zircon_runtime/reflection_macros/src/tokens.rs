use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Path, Type};

pub(crate) fn path_tokens(path: Path) -> TokenStream2 {
    quote!(#path)
}

pub(crate) fn script_host_type_ref_tokens(
    ty: &Type,
    value_kind: Option<TokenStream2>,
    type_name: Option<String>,
    trait_path: TokenStream2,
) -> TokenStream2 {
    match (value_kind, type_name) {
        (Some(value_kind), Some(type_name)) => quote! {
            ::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(#value_kind, #type_name)
        },
        (Some(value_kind), None) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.value_kind = #value_kind;
            type_ref
        }},
        (None, Some(type_name)) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.type_name = #type_name.to_string();
            type_ref
        }},
        (None, None) => quote! {
            <#ty as #trait_path>::script_host_type_ref()
        },
    }
}
