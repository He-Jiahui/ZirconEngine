use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_search_query(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("query")
        .and_then(value_as_string)
        .unwrap_or_default()
}
