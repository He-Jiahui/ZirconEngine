use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::shared::{alert_color_severity, app_bar_color, variant_contains};

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
                    app_bar_color(attributes).as_str(),
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
