use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Index, LitInt, Member, Type};

use crate::attributes::{parse_field_attributes, FieldAttributes};

pub(crate) struct ReflectedField {
    pub(crate) name: String,
    pub(crate) member: Option<Member>,
    pub(crate) ty: Option<Type>,
    pub(crate) value_type_path: String,
    pub(crate) editor_hint: syn::Ident,
    pub(crate) readonly: bool,
    pub(crate) serializable: bool,
    pub(crate) editor_visible: bool,
    pub(crate) read: Option<syn::Path>,
    pub(crate) write: Option<syn::Path>,
}

pub(crate) fn collect_virtual_fields(
    fields: Vec<FieldAttributes>,
) -> syn::Result<Vec<ReflectedField>> {
    let mut reflected = Vec::with_capacity(fields.len());
    for field in fields {
        let name = field.name.clone().ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "virtual reflected field is missing its name",
            )
        })?;
        let value_type_path = field.value_type_path.clone().ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "virtual reflected field is missing its value type path",
            )
        })?;
        reflected.push(build_reflected_field(
            name,
            None,
            None,
            value_type_path,
            field,
            proc_macro2::Span::call_site(),
        )?);
    }
    Ok(reflected)
}

pub(crate) fn collect_fields(fields: &syn::Fields) -> syn::Result<Vec<ReflectedField>> {
    let mut reflected = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let attributes = parse_field_attributes(&field.attrs)?;
        if attributes.skip {
            continue;
        }
        reflected.push(reflected_field(field, index, attributes)?);
    }
    Ok(reflected)
}

pub(crate) fn field_info_tokens(field: &ReflectedField) -> TokenStream {
    let name = &field.name;
    let value_type_path = &field.value_type_path;
    let editor_hint = &field.editor_hint;
    let editable = !field.readonly;
    let serializable = field.serializable;
    let editor_visible = field.editor_visible;
    quote! {
        ::zircon_runtime_interface::reflect::ReflectFieldInfo::new(
            #name,
            #value_type_path,
            ::zircon_runtime_interface::reflect::ReflectEditorHint::#editor_hint,
        )
        .with_editable(#editable)
        .with_serializable(#serializable)
        .with_editor_visible(#editor_visible)
    }
}

pub(crate) fn read_arm_tokens(field: &ReflectedField) -> TokenStream {
    let name = &field.name;
    if let Some(read) = &field.read {
        return quote!(#name => #read(self),);
    }
    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("virtual reflected fields require a read accessor"););
    };
    quote! {
        #name => Ok(
            <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::to_reflected_value(
                &self.#member,
            ),
        ),
    }
}

pub(crate) fn read_slot_arm_tokens(field: &ReflectedField, index: usize) -> TokenStream {
    let slot = slot_literal(index);
    if let Some(read) = &field.read {
        return quote!(#slot => #read(self),);
    }
    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("virtual reflected fields require a read accessor"););
    };
    quote! {
        #slot => Ok(
            <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::to_reflected_value(
                &self.#member,
            ),
        ),
    }
}

pub(crate) fn write_arm_tokens(field: &ReflectedField, type_path: &TokenStream) -> TokenStream {
    let name = &field.name;
    if field.readonly {
        return quote! {
            #name => Err(::zircon_runtime_interface::reflect::ReflectError::NonEditableField {
                type_path: (#type_path).to_string(),
                field_name: field_name.to_string(),
            })
        };
    }
    if let Some(write) = &field.write {
        return quote!(#name => #write(self, value));
    }

    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("editable virtual reflected fields require a write accessor"););
    };
    quote! {
        #name => {
            let next = <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::from_reflected_value(
                value,
                #type_path,
                field_name,
            )?;
            if self.#member == next {
                Ok(false)
            } else {
                self.#member = next;
                Ok(true)
            }
        }
    }
}

pub(crate) fn write_slot_arm_tokens(
    field: &ReflectedField,
    index: usize,
    type_path: &TokenStream,
) -> TokenStream {
    let slot = slot_literal(index);
    let name = &field.name;
    if field.readonly {
        return quote! {
            #slot => Err(::zircon_runtime_interface::reflect::ReflectError::NonEditableField {
                type_path: (#type_path).to_string(),
                field_name: #name.to_string(),
            })
        };
    }
    if let Some(write) = &field.write {
        return quote!(#slot => #write(self, value));
    }

    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("editable virtual reflected fields require a write accessor"););
    };
    quote! {
        #slot => {
            let next = <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::from_reflected_value(
                value,
                #type_path,
                #name,
            )?;
            if self.#member == next {
                Ok(false)
            } else {
                self.#member = next;
                Ok(true)
            }
        }
    }
}

fn slot_literal(index: usize) -> LitInt {
    LitInt::new(&format!("{index}u32"), proc_macro2::Span::call_site())
}

fn reflected_field(
    field: &Field,
    index: usize,
    attributes: FieldAttributes,
) -> syn::Result<ReflectedField> {
    let member = field
        .ident
        .clone()
        .map(Member::Named)
        .unwrap_or_else(|| Member::Unnamed(Index::from(index)));
    let name = attributes.name.clone().unwrap_or_else(|| match &member {
        Member::Named(ident) => ident.to_string(),
        Member::Unnamed(index) => index.index.to_string(),
    });
    if name.trim().is_empty() {
        return Err(syn::Error::new(
            field.span(),
            "reflected field name must not be empty",
        ));
    }
    let value_type_path = match attributes.value_type_path.clone() {
        Some(path) if !path.trim().is_empty() => path,
        Some(_) => {
            return Err(syn::Error::new(
                field.span(),
                "reflected field value_type_path must not be empty",
            ))
        }
        None => infer_value_type_path(&field.ty).ok_or_else(|| {
            syn::Error::new(
                field.ty.span(),
                "reflected field type requires value_type_path = \"...\"",
            )
        })?,
    };
    build_reflected_field(
        name,
        Some(member),
        Some(field.ty.clone()),
        value_type_path,
        attributes,
        field.span(),
    )
}

fn build_reflected_field(
    name: String,
    member: Option<Member>,
    ty: Option<Type>,
    value_type_path: String,
    attributes: FieldAttributes,
    span: proc_macro2::Span,
) -> syn::Result<ReflectedField> {
    let editor_hint_name = attributes
        .editor_hint
        .clone()
        .unwrap_or_else(|| inferred_editor_hint(&value_type_path).to_string());
    let editor_hint = parse_editor_hint(&editor_hint_name, span)?;
    Ok(ReflectedField {
        name,
        member,
        ty,
        value_type_path,
        editor_hint,
        readonly: attributes.readonly,
        serializable: attributes.serializable.unwrap_or(true),
        editor_visible: attributes.editor_visible.unwrap_or(true),
        read: attributes.read,
        write: attributes.write,
    })
}

fn infer_value_type_path(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => {
            let segment = path.path.segments.last()?;
            let value = match segment.ident.to_string().as_str() {
                "bool" => "Bool",
                "i8" | "i16" | "i32" | "i64" | "isize" => "Integer",
                "u8" | "u16" | "u32" | "u64" | "usize" => "Unsigned",
                "f32" | "Real" => "Scalar",
                "String" | "str" => "String",
                "Vec2" => "Vec2",
                "Vec3" => "Vec3",
                "Vec4" => "Vec4",
                "EntityId" => "Entity",
                "Vec" => "List",
                _ => return None,
            };
            Some(value.to_string())
        }
        _ => None,
    }
}

fn inferred_editor_hint(value_type_path: &str) -> &'static str {
    match value_type_path {
        "Bool" => "Bool",
        "Integer" => "Integer",
        "Unsigned" => "Unsigned",
        "Scalar" => "Scalar",
        "String" => "String",
        "Vec2" => "Vec2",
        "Vec3" => "Vec3",
        "Vec4" => "Vec4",
        "Enum" => "Enum",
        "Entity" => "Entity",
        "Resource" => "Resource",
        "Json" => "Json",
        _ => "None",
    }
}

fn parse_editor_hint(name: &str, span: proc_macro2::Span) -> syn::Result<syn::Ident> {
    const SUPPORTED: &[&str] = &[
        "None",
        "String",
        "MultilineString",
        "Bool",
        "Integer",
        "Unsigned",
        "Scalar",
        "Vec2",
        "Vec3",
        "Vec4",
        "Enum",
        "Entity",
        "Resource",
        "Color",
        "Json",
    ];
    if !SUPPORTED.contains(&name) {
        return Err(syn::Error::new(
            span,
            format!("unsupported reflection editor_hint `{name}`"),
        ));
    }
    Ok(syn::Ident::new(name, span))
}
