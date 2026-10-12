use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_media_source(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> String {
    attributes
        .get("image")
        .or_else(|| attributes.get("source"))
        .or_else(|| attributes.get("media"))
        .or_else(|| {
            if matches!(component_role, "image" | "svg-icon") {
                attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default()
}
