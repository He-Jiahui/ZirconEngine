use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};

pub(super) fn projected_selection_state(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("selection_state")
        .and_then(value_as_string)
        .or_else(|| {
            attributes
                .get("multiple")
                .and_then(value_as_bool)
                .map(|multiple| if multiple { "multi" } else { "single" }.to_string())
        })
        .unwrap_or_default()
}

pub(super) fn projected_selected(attributes: &BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("selected")
        .and_then(value_as_bool)
        .unwrap_or(false)
}
