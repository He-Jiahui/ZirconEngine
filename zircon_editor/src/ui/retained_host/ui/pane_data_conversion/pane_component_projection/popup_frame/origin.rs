use std::borrow::Cow;
use std::collections::BTreeMap;

use toml::Value;

pub(super) fn origin_axis<'a>(
    attributes: &'a BTreeMap<String, Value>,
    key: &str,
    default: &'a str,
) -> Cow<'a, str> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(Cow::Borrowed)
        .unwrap_or(Cow::Borrowed(default))
}

pub(super) fn default_anchor_origin_vertical(component_role: &str) -> &'static str {
    match component_role {
        "menu" | "context-menu" | "context-action-menu" | "dropdown-popup" => "bottom",
        _ => "top",
    }
}

pub(super) fn default_anchor_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

pub(super) fn default_transform_origin_vertical(_component_role: &str) -> &'static str {
    "top"
}

pub(super) fn default_transform_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

pub(super) fn origin_offset(length: f32, axis: &str) -> f32 {
    match axis {
        "center" => length * 0.5,
        "bottom" | "right" | "end" => length,
        value => value.parse::<f32>().unwrap_or(0.0),
    }
}

#[cfg(test)]
#[path = "tests/origin_optimization_tests.rs"]
mod optimization_tests;
