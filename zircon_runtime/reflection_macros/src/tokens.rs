//! 为派生字段和宿主函数签名生成脚本类型引用；显式覆盖值类别或名称时，未覆盖部分沿用宿主转换 trait。

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
    // 只提供一种 override 时沿用 trait 的另一项默认值；两项都给定时无需 trait 默认值。
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
