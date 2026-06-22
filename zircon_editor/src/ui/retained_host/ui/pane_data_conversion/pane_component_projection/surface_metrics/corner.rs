use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64};

pub(in super::super) fn projected_corner_radius(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
) -> f32 {
    attributes
        .get("corner_radius")
        .or_else(|| attributes.get("radius"))
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| match component_role {
            _ if attributes
                .get("square")
                .and_then(value_as_bool)
                .unwrap_or(false) =>
            {
                0.0
            }
            "alert" => 4.0,
            "card" | "paper" | "dialog" | "confirm-dialog" | "alert-dialog" | "popover"
            | "menu" | "tooltip" | "snackbar" | "snackbar-content" => 4.0,
            "app-bar" => 0.0,
            "drawer" => 0.0,
            _ => 0.0,
        })
}
