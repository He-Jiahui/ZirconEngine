use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_transition_kind(
    attributes: &BTreeMap<String, Value>,
    component_role: &str,
) -> String {
    attributes
        .get("transition_kind")
        .and_then(value_as_string)
        .or_else(|| attributes.get("transition").and_then(value_as_string))
        .or_else(|| transition_kind_from_role(component_role).map(str::to_string))
        .unwrap_or_default()
}

fn transition_kind_from_role(component_role: &str) -> Option<&'static str> {
    match component_role {
        "collapse" => Some("collapse"),
        "fade" => Some("fade"),
        "grow" => Some("grow"),
        "slide" => Some("slide"),
        "zoom" => Some("zoom"),
        _ => None,
    }
}
