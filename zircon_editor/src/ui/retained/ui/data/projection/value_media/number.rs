use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;

pub(super) fn projected_value_number(attributes: &BTreeMap<String, toml::Value>) -> f64 {
    attributes
        .get("value")
        .or_else(|| attributes.get("progress"))
        .or_else(|| attributes.get("dot_size"))
        .or_else(|| attributes.get("status_mark_size"))
        .or_else(|| attributes.get("arrow_size"))
        .or_else(|| attributes.get("track_width"))
        .or_else(|| attributes.get("icon_size"))
        .and_then(value_as_f64)
        .unwrap_or(0.0)
}
