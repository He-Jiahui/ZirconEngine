use std::collections::BTreeMap;

use toml::Value;

pub(super) fn first_non_empty_string_attribute(
    attributes: &BTreeMap<String, Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .filter_map(|key| string_attribute(attributes, key))
        .find(|value| !value.is_empty())
}

pub(super) fn string_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn bool_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    attributes.get(key).and_then(Value::as_bool)
}

pub(super) fn f32_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<f32> {
    attributes.get(key).and_then(|value| {
        value
            .as_float()
            .map(|value| value as f32)
            .or_else(|| value.as_integer().map(|value| value as f32))
    })
}
