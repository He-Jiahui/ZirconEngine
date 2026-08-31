use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::shared::{append_variant_token, pascal_case};

pub(super) fn append_chip_variant_tokens(
    attributes: &BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let chip_variant = attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "filled".to_string());
    append_variant_token(variant, &chip_variant);

    let size = attributes
        .get("size")
        .or_else(|| attributes.get("mui_size"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "medium".to_string());
    append_variant_token(variant, &size);
    append_variant_token(variant, &format!("size{}", pascal_case(&size)));

    let color = attributes
        .get("color")
        .or_else(|| attributes.get("mui_color"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    append_variant_token(variant, &color);
    append_variant_token(variant, &format!("color{}", pascal_case(&color)));

    if attributes
        .get("clickable")
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "clickable");
    }
    if chip_is_deletable(attributes) {
        append_variant_token(variant, "deletable");
        append_variant_token(variant, "hasDeleteIcon");
    }
    if borrowed_chip_string_attribute(attributes, &["deleteIcon", "delete_icon"])
        .is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasDeleteIcon");
    }
    if borrowed_chip_string_attribute(attributes, &["icon"]).is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasIcon");
    }
    if borrowed_chip_string_attribute(attributes, &["avatar"])
        .is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasAvatar");
    }
    if attributes
        .get("focusVisible")
        .or_else(|| attributes.get("focus_visible"))
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "focusVisible");
    }
}

fn borrowed_chip_string_attribute<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| attributes.get(*name))
        .and_then(toml::Value::as_str)
}

fn chip_is_deletable(attributes: &BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("deletable")
        .and_then(value_as_bool)
        .unwrap_or(false)
        || chip_has_non_empty_attribute(attributes, &["onDelete", "on_delete"])
}

fn chip_has_non_empty_attribute(
    attributes: &BTreeMap<String, toml::Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes.get(*name).is_some_and(|value| match value {
            toml::Value::String(value) => !value.trim().is_empty(),
            toml::Value::Boolean(value) => *value,
            toml::Value::Array(values) => !values.is_empty(),
            toml::Value::Table(values) => !values.is_empty(),
            toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Datetime(_) => true,
        })
    })
}

#[cfg(test)]
#[path = "chip/borrowed_presence_tests.rs"]
mod borrowed_presence_tests;
