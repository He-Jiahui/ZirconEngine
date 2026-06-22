use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn origin_axis(
    attributes: &BTreeMap<String, Value>,
    key: &str,
    default: &str,
) -> String {
    attributes
        .get(key)
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
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
