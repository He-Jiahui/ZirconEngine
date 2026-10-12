use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_transition_direction(
    attributes: &BTreeMap<String, Value>,
    kind: &str,
) -> String {
    attributes
        .get("transition_direction")
        .or_else(|| attributes.get("direction"))
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            if kind == "slide" {
                "down".to_string()
            } else {
                String::new()
            }
        })
}
