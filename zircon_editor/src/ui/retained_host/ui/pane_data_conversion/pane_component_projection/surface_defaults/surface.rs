use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;

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
    match borrowed_surface_app_bar_color(attributes) {
        "default" | "inherit" => "paper".to_string(),
        "transparent" => "transparent".to_string(),
        color => color.to_string(),
    }
}

fn borrowed_surface_app_bar_color(attributes: &BTreeMap<String, toml::Value>) -> &str {
    attributes
        .get("color")
        .and_then(toml::Value::as_str)
        .filter(|color| !color.is_empty())
        .unwrap_or("primary")
}

#[cfg(test)]
#[path = "surface/borrowed_app_bar_color_tests.rs"]
mod borrowed_app_bar_color_tests;
