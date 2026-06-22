use std::collections::BTreeMap;

use super::super::attribute_values::first_non_empty_string_attribute;

pub(in super::super) fn projected_dialog_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Option<String> {
    if !matches!(component_role, "dialog" | "confirm-dialog" | "alert-dialog") {
        return None;
    }
    first_non_empty_string_attribute(attributes, &["message", "description", "body"])
}
