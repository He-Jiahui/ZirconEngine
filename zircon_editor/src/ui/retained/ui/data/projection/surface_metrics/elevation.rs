use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64};

pub(in super::super) fn projected_elevation(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    attributes
        .get("elevation")
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| default_mui_elevation(attributes, component_role, component_variant))
}

fn default_mui_elevation(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    if component_variant
        .split_whitespace()
        .any(|part| part == "outlined")
    {
        return 0.0;
    }
    match component_role {
        "app-bar" => 4.0,
        "alert" => 0.0,
        "card"
            if attributes
                .get("raised")
                .and_then(value_as_bool)
                .unwrap_or(false) =>
        {
            8.0
        }
        "card" => 1.0,
        "paper" => 1.0,
        "dialog" | "confirm-dialog" | "alert-dialog" => 24.0,
        "popover" | "menu" => 8.0,
        "snackbar" | "snackbar-content" => 6.0,
        "drawer" => 16.0,
        _ => 0.0,
    }
}
