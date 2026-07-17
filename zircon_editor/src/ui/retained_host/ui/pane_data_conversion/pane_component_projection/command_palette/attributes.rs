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
        Value::String(value) => match value.trim() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn option_index(value: Option<&Value>) -> Option<usize> {
    value
        .and_then(Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
}
