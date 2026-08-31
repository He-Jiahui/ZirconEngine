use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::shared::{alert_color_severity, variant_contains};

pub(super) fn projected_text_tone(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    attributes
        .get("text_tone")
        .and_then(value_as_string)
        .unwrap_or_else(|| match component_role {
            "app-bar"
                if matches!(
                    borrowed_app_bar_color(attributes),
                    "inherit" | "transparent"
                ) =>
            {
                "primary".to_string()
            }
            "app-bar" => "inverse".to_string(),
            "alert" if variant_contains(component_variant, "filled") => "inverse".to_string(),
            "alert" => alert_color_severity(attributes),
            "tooltip" | "snackbar" | "snackbar-content" => "inverse".to_string(),
            _ => String::new(),
        })
}

fn borrowed_app_bar_color(attributes: &BTreeMap<String, toml::Value>) -> &str {
    attributes
        .get("color")
        .and_then(toml::Value::as_str)
        .filter(|color| !color.is_empty())
        .unwrap_or("primary")
}

#[cfg(test)]
#[path = "text_tone/borrowed_app_bar_color_tests.rs"]
mod borrowed_app_bar_color_tests;
