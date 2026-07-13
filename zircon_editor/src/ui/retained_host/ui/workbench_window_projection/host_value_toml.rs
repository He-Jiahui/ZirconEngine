use std::collections::BTreeMap;

use crate::ui::template_runtime::RetainedUiHostValue;

pub(super) fn toml_values_from_host_properties(
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> BTreeMap<String, toml::Value> {
    let mut values = properties
        .iter()
        .filter_map(|(key, value)| Some((key.clone(), toml_value_from_host_value(value)?)))
        .collect::<BTreeMap<_, _>>();
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    alias_toml_value_key(&mut values, "thumb_outline_color", "border_color");
    alias_toml_value_key(&mut values, "disabled_opacity", "opacity");
    values
}

fn alias_toml_value_key(values: &mut BTreeMap<String, toml::Value>, source: &str, target: &str) {
    if values.contains_key(target) {
        return;
    }
    if let Some(value) = values.get(source).cloned() {
        values.insert(target.to_string(), value);
    }
}

fn toml_value_from_host_value(value: &RetainedUiHostValue) -> Option<toml::Value> {
    match value {
        RetainedUiHostValue::String(value) => Some(toml::Value::String(value.clone())),
        RetainedUiHostValue::Integer(value) => Some(toml::Value::Integer(*value)),
        RetainedUiHostValue::Float(value) => Some(toml::Value::Float(*value)),
        RetainedUiHostValue::Bool(value) => Some(toml::Value::Boolean(*value)),
        RetainedUiHostValue::Datetime(value) => value.parse().ok().map(toml::Value::Datetime),
        RetainedUiHostValue::Array(values) => Some(toml::Value::Array(
            values
                .iter()
                .filter_map(toml_value_from_host_value)
                .collect(),
        )),
        RetainedUiHostValue::Table(values) => Some(toml::Value::Table(
            values
                .iter()
                .filter_map(|(key, value)| Some((key.clone(), toml_value_from_host_value(value)?)))
                .collect(),
        )),
    }
}
