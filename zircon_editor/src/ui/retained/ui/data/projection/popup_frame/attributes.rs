use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_f64;

pub(super) fn float_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<f32> {
    attributes
        .get(key)
        .and_then(value_as_f64)
        .map(|value| value as f32)
}
