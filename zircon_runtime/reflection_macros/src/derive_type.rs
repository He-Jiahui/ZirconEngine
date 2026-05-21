use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use crate::attrs::{parse_field_attrs, parse_script_type_attrs};
use crate::tokens::{path_tokens, script_host_type_ref_tokens};

pub(crate) fn derive_zircon_script_type_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "ZirconScriptType does not support generic parameters",
        ));
    }
    let args = parse_script_type_attrs(&input.attrs)?;
    let type_name = args.name.unwrap_or_else(|| ident.to_string());
    let value_kind = args.value_kind.map(path_tokens).unwrap_or_else(|| {
        quote!(::zircon_runtime::core::framework::script::ScriptHostValueKind::Null)
    });
    let allow_value_construction = args.allow_value_construction.unwrap_or(false);
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let (fields, default_prototype) = match input.data {
        Data::Struct(data) => (
            field_descriptor_tokens(&data.fields)?,
            quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Struct),
        ),
        Data::Enum(_) => (
            Vec::new(),
            quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Enum),
        ),
        Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "ZirconScriptType does not support unions",
            ))
        }
    };
    let prototype = args.prototype.map(path_tokens).unwrap_or(default_prototype);

    Ok(quote! {
        impl ::zircon_runtime::core::framework::script::ZirconScriptType for #ident {
            fn script_host_type_descriptor() -> ::zircon_runtime::core::framework::script::ScriptHostTypeDescriptor {
                ::zircon_runtime::core::framework::script::ScriptHostTypeDescriptor::new(
                    #type_name,
                    #value_kind,
                )
                .with_type_ref(::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(#value_kind, #type_name))
                .with_prototype_kind(#prototype)
                .allow_value_construction(#allow_value_construction)
                #(#fields)*
                #documentation
            }
        }
    })
}

fn field_descriptor_tokens(fields: &Fields) -> syn::Result<Vec<TokenStream2>> {
    let mut descriptors = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let args = parse_field_attrs(&field.attrs)?;
        if args.skip {
            continue;
        }
        let field_name = match (&args.name, &field.ident) {
            (Some(name), _) => name.clone(),
            (None, Some(ident)) => ident.to_string(),
            (None, None) => index.to_string(),
        };
        let field_type = &field.ty;
        let type_ref = script_host_type_ref_tokens(
            field_type,
            args.value_kind.map(path_tokens),
            args.type_name,
            quote!(::zircon_runtime::core::framework::script::ScriptHostFromValue),
        );
        let documentation = args
            .documentation
            .map(|doc| quote!(.with_documentation(#doc)));
        descriptors.push(quote! {
            .with_field({
                let type_ref = #type_ref;
                ::zircon_runtime::core::framework::script::ScriptHostFieldDescriptor::new(#field_name, type_ref.value_kind)
                    .with_type_ref(type_ref)
                    #documentation
            })
        });
    }
    Ok(descriptors)
}
