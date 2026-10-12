use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_float_array;

pub(super) fn projected_vector_components(attributes: &BTreeMap<String, toml::Value>) -> Vec<f32> {
    attributes
        .get("value")
        .and_then(value_as_float_array)
        .unwrap_or_default()
}
