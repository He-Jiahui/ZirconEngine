use syn::{Attribute, Ident, Path};

use crate::args::{parse_lit_bool, parse_lit_str, FieldArgs, ScriptTypeArgs};

pub(crate) fn parse_script_type_attrs(attrs: &[Attribute]) -> syn::Result<ScriptTypeArgs> {
    let mut args = ScriptTypeArgs::default();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("zircon_script"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                args.name = Some(parse_lit_str(meta)?);
            } else if meta.path.is_ident("value_kind") {
                args.value_kind = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("prototype") {
                args.prototype = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("allow_value_construction") {
                args.allow_value_construction = Some(parse_lit_bool(meta)?);
            } else if meta.path.is_ident("documentation") || meta.path.is_ident("doc") {
                args.documentation = Some(parse_lit_str(meta)?);
            } else {
                return Err(meta.error("unknown zircon_script type key"));
            }
            Ok(())
        })?;
    }
    Ok(args)
}

pub(crate) fn parse_field_attrs(attrs: &[Attribute]) -> syn::Result<FieldArgs> {
    let mut args = FieldArgs::default();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("zircon_script"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                args.name = Some(parse_lit_str(meta)?);
            } else if meta.path.is_ident("type_name") {
                args.type_name = Some(parse_lit_str(meta)?);
            } else if meta.path.is_ident("value_kind") {
                args.value_kind = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("documentation") || meta.path.is_ident("doc") {
                args.documentation = Some(parse_lit_str(meta)?);
            } else if meta.path.is_ident("skip") {
                args.skip = true;
            } else {
                return Err(meta.error("unknown zircon_script field key"));
            }
            Ok(())
        })?;
    }
    Ok(args)
}

pub(crate) fn host_attr_function_ident(item: &syn::Item) -> Option<Ident> {
    match item {
        syn::Item::Fn(function)
            if has_attr_with_last_segment(&function.attrs, "zircon_host_function") =>
        {
            Some(function.sig.ident.clone())
        }
        _ => None,
    }
}

pub(crate) fn script_type_ident(item: &syn::Item) -> Option<Ident> {
    match item {
        syn::Item::Struct(item) if derives_zircon_script_type(&item.attrs) => {
            Some(item.ident.clone())
        }
        syn::Item::Enum(item) if derives_zircon_script_type(&item.attrs) => {
            Some(item.ident.clone())
        }
        _ => None,
    }
}

fn has_attr_with_last_segment(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|attr| path_last_segment_is(attr.path(), name))
}

fn path_last_segment_is(path: &Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn derives_zircon_script_type(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if path_last_segment_is(&meta.path, "ZirconScriptType") {
                found = true;
            }
            Ok(())
        });
        found
    })
}
