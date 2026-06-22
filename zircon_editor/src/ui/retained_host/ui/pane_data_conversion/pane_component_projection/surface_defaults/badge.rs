use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::shared::{append_variant_token, pascal_case, string_from_toml_map};

pub(super) fn append_badge_variant_tokens(
    attributes: &BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let badge_variant = badge_variant(attributes);
    append_variant_token(variant, &badge_variant);
    if badge_is_invisible(attributes, &badge_variant) {
        append_variant_token(variant, "invisible");
    }

    let color = attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    append_variant_token(variant, &color);

    let overlap = attributes
        .get("overlap")
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "rectangular".to_string());
    append_variant_token(variant, &overlap);
    append_variant_token(variant, &format!("overlap{}", pascal_case(&overlap)));

    let (vertical, horizontal) = badge_anchor_origin(attributes);
    append_variant_token(variant, &vertical);
    append_variant_token(variant, &horizontal);
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal)
        ),
    );
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal),
            pascal_case(&overlap)
        ),
    );
}

fn badge_variant(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string())
}

fn badge_is_invisible(attributes: &BTreeMap<String, toml::Value>, variant: &str) -> bool {
    if attributes
        .get("invisible")
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        return true;
    }
    let content = attributes
        .get("badgeContent")
        .or_else(|| attributes.get("badge_content"))
        .or_else(|| attributes.get("value_text"));
    if variant != "dot" && !content.is_some_and(badge_content_present) {
        return true;
    }
    content.is_some_and(|value| {
        badge_content_is_numeric_zero(value)
            && !attributes
                .get("showZero")
                .or_else(|| attributes.get("show_zero"))
                .and_then(value_as_bool)
                .unwrap_or(false)
    })
}

fn badge_content_present(value: &toml::Value) -> bool {
    match value {
        toml::Value::String(value) => !value.trim().is_empty(),
        toml::Value::Array(values) => !values.is_empty(),
        toml::Value::Table(values) => !values.is_empty(),
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => true,
    }
}

fn badge_content_is_numeric_zero(value: &toml::Value) -> bool {
    match value {
        toml::Value::Integer(value) => *value == 0,
        toml::Value::Float(value) => *value == 0.0,
        _ => false,
    }
}

fn badge_anchor_origin(attributes: &BTreeMap<String, toml::Value>) -> (String, String) {
    let anchor_origin = attributes.get("anchorOrigin");
    let vertical = string_from_toml_map(anchor_origin, "vertical")
        .or_else(|| {
            attributes
                .get("anchor_origin_vertical")
                .and_then(value_as_string)
        })
        .unwrap_or_else(|| "top".to_string());
    let horizontal = string_from_toml_map(anchor_origin, "horizontal")
        .or_else(|| {
            attributes
                .get("anchor_origin_horizontal")
                .and_then(value_as_string)
        })
        .unwrap_or_else(|| "right".to_string());
    (vertical, horizontal)
}
