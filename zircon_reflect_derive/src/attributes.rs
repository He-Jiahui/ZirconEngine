use syn::spanned::Spanned;
use syn::{Attribute, LitBool, LitStr, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SerializationKind {
    None,
    Value,
    Json,
    ResourceHandle,
    EntityReference,
}

impl Default for SerializationKind {
    fn default() -> Self {
        Self::Value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ScriptVisibilityKind {
    #[default]
    Private,
    Public,
}

#[derive(Default)]
pub(crate) struct ContainerAttributes {
    pub(crate) component: bool,
    pub(crate) resource: bool,
    pub(crate) type_path: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) plugin_owned: bool,
    pub(crate) serializable: Option<bool>,
    pub(crate) editor_visible: Option<bool>,
    pub(crate) remote_visible: Option<bool>,
    pub(crate) script_visibility: ScriptVisibilityKind,
    pub(crate) serialization: SerializationKind,
    pub(crate) virtual_fields: Vec<FieldAttributes>,
}

#[derive(Default)]
pub(crate) struct FieldAttributes {
    pub(crate) skip: bool,
    pub(crate) name: Option<String>,
    pub(crate) value_type_path: Option<String>,
    pub(crate) editor_hint: Option<String>,
    pub(crate) readonly: bool,
    pub(crate) serializable: Option<bool>,
    pub(crate) editor_visible: Option<bool>,
    pub(crate) read: Option<Path>,
    pub(crate) write: Option<Path>,
}

pub(crate) fn parse_container_attributes(
    attributes: &[Attribute],
) -> syn::Result<ContainerAttributes> {
    let mut parsed = ContainerAttributes::default();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("zr_reflect"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("component") {
                parsed.component = true;
            } else if meta.path.is_ident("resource") {
                parsed.resource = true;
            } else if meta.path.is_ident("type_path") {
                parsed.type_path = Some(parse_string(meta)?);
            } else if meta.path.is_ident("display_name") {
                parsed.display_name = Some(parse_string(meta)?);
            } else if meta.path.is_ident("plugin_owned") {
                parsed.plugin_owned = parse_bool_or_true(meta)?;
            } else if meta.path.is_ident("serializable") {
                parsed.serializable = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("editor_visible") {
                parsed.editor_visible = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("remote_visible") {
                parsed.remote_visible = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("script_visibility") {
                let span = meta.path.span();
                parsed.script_visibility = match parse_string(meta)?.as_str() {
                    "public" => ScriptVisibilityKind::Public,
                    "private" => ScriptVisibilityKind::Private,
                    other => {
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "unsupported script_visibility `{other}`; expected `public` or `private`"
                            ),
                        ))
                    }
                };
            } else if meta.path.is_ident("serialization") {
                parsed.serialization = parse_serialization(meta)?;
            } else if meta.path.is_ident("field") {
                let mut field = FieldAttributes::default();
                meta.parse_nested_meta(|nested| parse_field_meta(&mut field, nested))?;
                validate_field_attributes(&field)?;
                if field.name.is_none() || field.value_type_path.is_none() || field.read.is_none() {
                    return Err(meta.error(
                        "virtual reflected fields require name, value_type_path, and read",
                    ));
                }
                if !field.readonly && field.write.is_none() {
                    return Err(meta.error(
                        "editable virtual reflected fields require a write accessor",
                    ));
                }
                parsed.virtual_fields.push(field);
            } else {
                return Err(meta.error("unknown zr_reflect type attribute"));
            }
            Ok(())
        })?;
    }

    if parsed.component && parsed.resource {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "a reflected type cannot be both a component and a resource",
        ));
    }
    if parsed
        .type_path
        .as_ref()
        .is_some_and(|path| path.trim().is_empty())
    {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "zr_reflect type_path must not be empty",
        ));
    }

    Ok(parsed)
}

pub(crate) fn parse_field_attributes(attributes: &[Attribute]) -> syn::Result<FieldAttributes> {
    let mut parsed = FieldAttributes::default();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("zr_reflect"))
    {
        attribute.parse_nested_meta(|meta| parse_field_meta(&mut parsed, meta))?;
    }

    validate_field_attributes(&parsed)?;
    Ok(parsed)
}

fn parse_field_meta(
    parsed: &mut FieldAttributes,
    meta: syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<()> {
    if meta.path.is_ident("skip") {
        parsed.skip = true;
    } else if meta.path.is_ident("name") {
        parsed.name = Some(parse_string(meta)?);
    } else if meta.path.is_ident("value_type_path") {
        parsed.value_type_path = Some(parse_string(meta)?);
    } else if meta.path.is_ident("editor_hint") {
        parsed.editor_hint = Some(parse_string(meta)?);
    } else if meta.path.is_ident("readonly") {
        parsed.readonly = true;
    } else if meta.path.is_ident("serializable") {
        parsed.serializable = Some(parse_bool(meta)?);
    } else if meta.path.is_ident("editor_visible") {
        parsed.editor_visible = Some(parse_bool(meta)?);
    } else if meta.path.is_ident("read") {
        parsed.read = Some(parse_path(meta)?);
    } else if meta.path.is_ident("write") {
        parsed.write = Some(parse_path(meta)?);
    } else {
        return Err(meta.error("unknown zr_reflect field attribute"));
    }
    Ok(())
}

fn validate_field_attributes(parsed: &FieldAttributes) -> syn::Result<()> {
    if parsed.readonly && parsed.write.is_some() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "a readonly reflected field cannot declare a write accessor",
        ));
    }
    Ok(())
}

fn parse_string(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    Ok(meta.value()?.parse::<LitStr>()?.value())
}

fn parse_bool(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    Ok(meta.value()?.parse::<LitBool>()?.value())
}

fn parse_bool_or_true(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.input.is_empty() {
        return Ok(true);
    }
    parse_bool(meta)
}

fn parse_path(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<Path> {
    let value = meta.value()?.parse::<LitStr>()?;
    value.parse()
}

fn parse_serialization(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<SerializationKind> {
    let value = parse_string(meta)?;
    match value.as_str() {
        "none" => Ok(SerializationKind::None),
        "value" => Ok(SerializationKind::Value),
        "json" => Ok(SerializationKind::Json),
        "resource_handle" => Ok(SerializationKind::ResourceHandle),
        "entity_reference" => Ok(SerializationKind::EntityReference),
        other => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unsupported reflection serialization `{other}`"),
        )),
    }
}
