use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;

pub(super) fn f32_attribute(
    attributes: &BTreeMap<String, toml::Value>,
    name: &str,
    default_value: f32,
) -> f32 {
    attributes
        .get(name)
        .and_then(value_as_f64)
        .unwrap_or(f64::from(default_value)) as f32
}
