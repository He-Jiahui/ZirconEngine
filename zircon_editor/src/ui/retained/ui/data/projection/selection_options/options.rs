use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_options;
pub(super) fn projected_options(attributes: &BTreeMap<String, toml::Value>) -> Vec<String> {
    attributes
        .get("options")
        .and_then(value_as_options)
        .unwrap_or_default()
}
