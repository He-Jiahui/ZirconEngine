use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::shared::{alert_color_severity, dialog_severity};

pub(super) fn projected_validation_level(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    disabled: bool,
    has_component_descriptor: bool,
) -> String {
    if let Some(level) = attributes
        .get("validation_level")
        .and_then(value_as_string)
        .filter(|level| !level.is_empty())
    {
        return level;
    }
    if disabled {
        return "disabled".to_string();
    }
    if component_role == "alert" {
        return alert_color_severity(attributes);
    }
    if matches!(component_role, "confirm-dialog" | "alert-dialog") {
        return dialog_severity(attributes);
    }
    if has_component_descriptor {
        "normal".to_string()
    } else {
        String::new()
    }
}
