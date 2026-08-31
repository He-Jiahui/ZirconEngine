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
    let type_identity = args.identity.unwrap_or_else(|| type_name.clone());
    if type_identity.is_empty() || type_identity.trim() != type_identity {
        return Err(syn::Error::new(
            ident.span(),
            "zircon_script type identity must be non-empty and already trimmed",
        ));
    }
    let value_kind = args.value_kind.map(path_tokens).unwrap_or_else(|| {
        quote!(::zircon_runtime::core::framework::script::ScriptHostValueKind::Null)
    });
    let allow_value_construction = args.allow_value_construction.unwrap_or(false);
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let (type_info, fields, default_prototype) = match input.data {
        Data::Struct(data) => {
            let (registrations, projections) = field_tokens(&data.fields, &type_identity)?;
            (
                quote!(::zircon_runtime::core::framework::script::__reflect::ReflectTypeInfo::struct_with_fields(
                    vec![#(#registrations),*]
                )),
                projections,
                quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Struct),
            )
        }
        Data::Enum(_) => (
            quote!(
                ::zircon_runtime::core::framework::script::__reflect::ReflectTypeInfo::new(
                    ::zircon_runtime::core::framework::script::__reflect::ReflectTypeKind::Enum,
                    Vec::new(),
                )
            ),
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
            fn reflect_type_registration() -> Result<
                ::zircon_runtime::core::framework::script::__reflect::ReflectTypeRegistration,
                ::zircon_runtime::core::framework::script::__reflect::ReflectError,
            > {
                let type_path = ::zircon_runtime::core::framework::script::__reflect::ReflectTypePath::new(
                    concat!(module_path!(), "::", stringify!(#ident)),
                    #type_name,
                )?;
                Ok(::zircon_runtime::core::framework::script::__reflect::ReflectTypeRegistration::new(
                    type_path,
                    #type_name,
                    #type_info,
                    ::zircon_runtime::core::framework::script::__reflect::ReflectSerializationStrategy::None,
                )
                .with_serializable(false)
                .with_editor_visible(false)
                .with_script_visibility(
                    ::zircon_runtime::core::framework::script::__reflect::ReflectScriptVisibility::Public,
                )
                #documentation)
            }

            fn script_host_type_projection() -> ::zircon_runtime::core::framework::script::ScriptHostTypeProjection {
                ::zircon_runtime::core::framework::script::ScriptHostTypeProjection::new(#value_kind)
                .with_prototype_kind(#prototype)
                .allow_value_construction(#allow_value_construction)
                #(#fields)*
            }
        }
    })
}

fn field_tokens(
    fields: &Fields,
    type_identity: &str,
) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>)> {
    let mut registrations = Vec::new();
    let mut projections = Vec::new();
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
        let field_identity = args.identity.unwrap_or_else(|| field_name.clone());
        let aliases = args.aliases;
        let alias_tokens = aliases.iter().map(|alias| quote!(#alias.to_string()));
        if field_identity.is_empty() || field_identity.trim() != field_identity {
            return Err(syn::Error::new(
                field.span(),
                "zircon_script field identity must be non-empty and already trimmed",
            ));
        }
        let field_type = &field.ty;
        let registration_type_ref = script_host_type_ref_tokens(
            field_type,
            args.value_kind.clone().map(path_tokens),
            args.type_name.clone(),
            quote!(::zircon_runtime::core::framework::script::ScriptHostFromArgument),
        );
        let projection_type_ref = registration_type_ref.clone();
        let documentation = args
            .documentation
            .map(|doc| quote!(.with_documentation(#doc)));
        registrations.push(quote! {{
            let type_ref = #registration_type_ref;
            ::zircon_runtime::core::framework::script::__reflect::ReflectFieldInfo::new(
                ::zircon_runtime::core::framework::script::__reflect::ReflectFieldId::from_stable_keys(
                    #type_identity,
                    #field_identity,
                ),
                #field_name,
                type_ref.type_name,
                ::zircon_runtime::core::framework::script::__reflect::ReflectEditorHint::None,
            )
            .with_aliases(vec![#(#alias_tokens),*])
            .with_serializable(false)
            .with_editor_visible(false)
            #documentation
        }});
        projections.push(quote! {
            .with_field({
                let type_ref = #projection_type_ref;
                ::zircon_runtime::core::framework::script::ScriptHostFieldProjection::new(
                    #field_name,
                    type_ref.value_kind,
                )
            })
        });
    }
    Ok((registrations, projections))
}
