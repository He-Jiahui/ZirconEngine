use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::shared::app_bar_color;

pub(super) fn projected_surface_variant(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    attributes
        .get("surface_variant")
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            default_mui_surface_variant(attributes, component_role, component_variant)
        })
}

fn default_mui_surface_variant(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    match component_role {
        "alert" => "alert".to_string(),
        "app-bar" => app_bar_surface_variant(attributes),
        "card"
            if component_variant
                .split_whitespace()
                .any(|part| part == "outlined") =>
        {
            "paper-outlined".to_string()
        }
        "card" => "paper".to_string(),
        "tooltip" => "tooltip".to_string(),
        "snackbar" | "snackbar-content" => "snackbar".to_string(),
        "paper"
            if component_variant
                .split_whitespace()
                .any(|part| part == "outlined") =>
        {
            "paper-outlined".to_string()
        }
        "paper" | "dialog" | "confirm-dialog" | "alert-dialog" | "popover" | "menu" => {
            "popup".to_string()
        }
        "drawer" => "paper".to_string(),
        _ => String::new(),
    }
}

fn app_bar_surface_variant(attributes: &BTreeMap<String, toml::Value>) -> String {
    match app_bar_color(attributes).as_str() {
        "default" | "inherit" => "paper".to_string(),
        "transparent" => "transparent".to_string(),
        color => color.to_string(),
    }
}
