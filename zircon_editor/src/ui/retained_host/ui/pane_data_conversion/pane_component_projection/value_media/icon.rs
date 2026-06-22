use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_icon_name(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> String {
    attributes
        .get("icon")
        .or_else(|| {
            if component_role == "icon" {
                attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default()
}
