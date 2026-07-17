use std::collections::BTreeMap;

use toml::Value;

pub(super) fn first_string_value(
    values: &toml::map::Map<String, Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

pub(super) fn string_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    string_attribute_ref(attributes, key).map(str::to_string)
}

pub(super) fn string_attribute_ref<'a>(
    attributes: &'a BTreeMap<String, Value>,
    key: &str,
) -> Option<&'a str> {
    attributes.get(key).and_then(string_value_ref)
}

fn string_value(value: &Value) -> Option<String> {
    string_value_ref(value).map(str::to_string)
}

fn string_value_ref(value: &Value) -> Option<&str> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn bool_value(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => string_bool(value),
        _ => None,
    }
}

pub(super) fn string_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

pub(super) fn usize_attribute(value: Option<&Value>) -> Option<usize> {
    match value? {
        Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    }
}

pub(super) fn normalized_tone(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "success" | "ok" | "done" => "success",
        "warning" | "warn" => "warning",
        "error" | "danger" | "failed" | "failure" => "error",
        _ => "info",
    }
    .to_string()
}
