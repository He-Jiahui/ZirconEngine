use std::collections::BTreeSet;

use toml::Value;

use super::attributes::first_string_value;

pub(super) fn command_id_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .map(command_id_values)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

pub(super) fn command_id_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(values) => values
            .iter()
            .flat_map(command_id_values)
            .filter(|value| !value.is_empty())
            .collect(),
        Value::String(value) => vec![value.split('|').next().unwrap_or(value).trim().to_string()],
        Value::Table(values) => {
            first_string_value(values, &["id", "command_id", "commandId", "value", "key"])
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}
