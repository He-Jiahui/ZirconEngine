use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

use crate::attributes::{parse_container_attributes, ScriptVisibilityKind, SerializationKind};
use crate::fields::{
    collect_fields, collect_virtual_fields, field_info_tokens, read_arm_tokens,
    read_slot_arm_tokens, write_arm_tokens, write_slot_arm_tokens,
};

pub(crate) fn derive_zr_reflect_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "ZrReflect does not support generic parameters",
        ));
    }

    let attributes = parse_container_attributes(&input.attrs)?;
    let ident = input.ident;
    let short_type_path = ident.to_string();
    let display_name = attributes
        .display_name
        .unwrap_or_else(|| short_type_path.clone());
    let type_path = attributes
        .type_path
        .clone()
        .map(|path| quote!(#path))
        .unwrap_or_else(|| quote!(concat!(module_path!(), "::", stringify!(#ident))));
    let type_identity = attributes
        .identity
        .map(|identity| quote!(#identity))
        .unwrap_or_else(|| type_path.clone());

    let (kind, mut fields) = match input.data {
        Data::Struct(data) => (
            quote!(::zircon_runtime_interface::reflect::ReflectTypeKind::Struct),
            collect_fields(&data.fields)?,
        ),
        Data::Enum(_) => (
            quote!(::zircon_runtime_interface::reflect::ReflectTypeKind::Enum),
            Vec::new(),
        ),
        Data::Union(union) => {
            return Err(syn::Error::new(
                union.union_token.span(),
                "ZrReflect does not support unions",
            ))
        }
    };
    fields.extend(collect_virtual_fields(attributes.virtual_fields)?);
    validate_unique_field_names(&fields, input_span)?;

    let field_info = fields
        .iter()
        .map(|field| field_info_tokens(field, &type_identity));
    let reflected_fields = if fields.is_empty() {
        quote!(::std::vec::Vec::new())
    } else {
        quote!(vec![#(#field_info),*])
    };
    let read_arms = fields.iter().map(read_arm_tokens);
    let read_slot_arms = fields
        .iter()
        .enumerate()
        .map(|(index, field)| read_slot_arm_tokens(field, index));
    let write_arms = fields
        .iter()
        .map(|field| write_arm_tokens(field, &type_path));
    let write_slot_arms = fields
        .iter()
        .enumerate()
        .map(|(index, field)| write_slot_arm_tokens(field, index, &type_path));
    let serialization = serialization_tokens(attributes.serialization);
    let component = attributes.component.then(|| quote!(.as_component()));
    let resource = attributes.resource.then(|| quote!(.as_resource()));
    let serializable = attributes
        .serializable
        .unwrap_or(!matches!(attributes.serialization, SerializationKind::None));
    let editor_visible = attributes.editor_visible.unwrap_or(true);
    let remote_visible = attributes.remote_visible.unwrap_or(false);
    let script_visibility = script_visibility_tokens(attributes.script_visibility);

    Ok(quote! {
        impl ::zircon_runtime_interface::reflect::ZrReflect for #ident {
            fn reflect_type_registration(
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectTypeRegistration,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                let type_path = ::zircon_runtime_interface::reflect::ReflectTypePath::new(
                    #type_path,
                    #short_type_path,
                )?;
                let fields = #reflected_fields;
                Ok(::zircon_runtime_interface::reflect::ReflectTypeRegistration::new(
                    type_path,
                    #display_name,
                    ::zircon_runtime_interface::reflect::ReflectTypeInfo::new(#kind, fields),
                    #serialization,
                )
                #component
                #resource
                .with_serializable(#serializable)
                .with_editor_visible(#editor_visible)
                .with_remote_visible(#remote_visible)
                .with_script_visibility(#script_visibility))
            }

            fn read_reflected_field(
                &self,
                field_name: &str,
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectedValue,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                match field_name {
                    #(#read_arms)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: field_name.to_string(),
                    }),
                }
            }

            fn write_reflected_field(
                &mut self,
                field_name: &str,
                value: ::zircon_runtime_interface::reflect::ReflectedValue,
            ) -> Result<bool, ::zircon_runtime_interface::reflect::ReflectError> {
                match field_name {
                    #(#write_arms,)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: field_name.to_string(),
                    }),
                }
            }

            fn read_reflected_field_by_slot(
                &self,
                field_slot: u32,
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectedValue,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                match field_slot {
                    #(#read_slot_arms)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: format!("#{field_slot}"),
                    }),
                }
            }

            fn write_reflected_field_by_slot(
                &mut self,
                field_slot: u32,
                value: ::zircon_runtime_interface::reflect::ReflectedValue,
            ) -> Result<bool, ::zircon_runtime_interface::reflect::ReflectError> {
                match field_slot {
                    #(#write_slot_arms,)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: format!("#{field_slot}"),
                    }),
                }
            }
        }
    })
}

fn validate_unique_field_names(
    fields: &[crate::fields::ReflectedField],
    span: proc_macro2::Span,
) -> syn::Result<()> {
    let mut names = BTreeSet::new();
    for field in fields {
        if !names.insert(field.name.as_str()) {
            return Err(syn::Error::new(
                span,
                format!("duplicate reflected field name `{}`", field.name),
            ));
        }
    }
    Ok(())
}

fn serialization_tokens(kind: SerializationKind) -> TokenStream {
    let variant = match kind {
        SerializationKind::None => quote!(None),
        SerializationKind::Value => quote!(Value),
        SerializationKind::Json => quote!(Json),
        SerializationKind::ResourceHandle => quote!(ResourceHandle),
        SerializationKind::EntityReference => quote!(EntityReference),
    };
    quote!(::zircon_runtime_interface::reflect::ReflectSerializationStrategy::#variant)
}

fn script_visibility_tokens(kind: ScriptVisibilityKind) -> TokenStream {
    let variant = match kind {
        ScriptVisibilityKind::Private => quote!(Private),
        ScriptVisibilityKind::Public => quote!(Public),
    };
    quote!(::zircon_runtime_interface::reflect::ReflectScriptVisibility::#variant)
}
