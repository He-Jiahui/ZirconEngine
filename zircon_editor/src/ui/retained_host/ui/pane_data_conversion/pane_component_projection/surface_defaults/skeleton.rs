use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::shared::{append_variant_token, has_non_empty_attribute, variant_contains};

pub(super) fn append_skeleton_variant_tokens(
    attributes: &BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    if !variant_has_any_token(variant, &["text", "rectangular", "rounded", "circular"]) {
        append_variant_token(variant, skeleton_shape_variant(attributes).as_str());
    }
    if let Some(animation) = skeleton_animation(attributes, variant) {
        append_variant_token(variant, animation);
    }
    if skeleton_has_children(attributes) {
        append_variant_token(variant, "withChildren");
        if !has_non_empty_attribute(attributes, &["width"]) {
            append_variant_token(variant, "fitContent");
        }
        if !has_non_empty_attribute(attributes, &["height"]) {
            append_variant_token(variant, "heightAuto");
        }
    }
}

fn skeleton_shape_variant(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "text".to_string())
}

fn skeleton_animation<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    variant: &str,
) -> Option<&'a str> {
    if variant_has_any_token(variant, &["pulse", "wave"]) {
        return None;
    }
    match attributes.get("animation") {
        Some(toml::Value::Boolean(false)) => None,
        Some(value) => value
            .as_str()
            .filter(|value| !value.is_empty() && *value != "false"),
        None => Some("pulse"),
    }
}

fn skeleton_has_children(attributes: &BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("hasChildren")
        .or_else(|| attributes.get("has_children"))
        .or_else(|| attributes.get("withChildren"))
        .or_else(|| attributes.get("with_children"))
        .and_then(value_as_bool)
        .unwrap_or(false)
}

fn variant_has_any_token(variant: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|token| variant_contains(variant, token))
}

#[cfg(test)]
#[path = "skeleton/borrowed_animation_tests.rs"]
mod borrowed_animation_tests;
