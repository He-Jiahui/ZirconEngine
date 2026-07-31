use std::collections::BTreeMap;

use crate::ui::template_runtime::RetainedUiHostValue;

pub(super) fn toml_values_from_host_properties(
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> BTreeMap<String, toml::Value> {
    toml_values_from_host_properties_filtered(properties, false)
}

pub(super) fn toml_values_from_host_properties_without_notifications(
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> BTreeMap<String, toml::Value> {
    toml_values_from_host_properties_filtered(properties, true)
}

fn toml_values_from_host_properties_filtered(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    exclude_notifications: bool,
) -> BTreeMap<String, toml::Value> {
    let mut values = properties
        .iter()
        .filter(|(key, _)| !exclude_notifications || key.as_str() != "notifications")
        .filter_map(|(key, value)| {
            Some((
                key.clone(),
                toml_value_from_host_value(value, key == "notifications")?,
            ))
        })
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

fn toml_value_from_host_value(
    value: &RetainedUiHostValue,
    notification_value: bool,
) -> Option<toml::Value> {
    match value {
        RetainedUiHostValue::String(value) => {
            record_notification_text_copy(notification_value);
            Some(toml::Value::String(value.clone()))
        }
        RetainedUiHostValue::Integer(value) => Some(toml::Value::Integer(*value)),
        RetainedUiHostValue::Float(value) => Some(toml::Value::Float(*value)),
        RetainedUiHostValue::Bool(value) => Some(toml::Value::Boolean(*value)),
        RetainedUiHostValue::Datetime(value) => value.parse().ok().map(toml::Value::Datetime),
        RetainedUiHostValue::Array(values) => Some(toml::Value::Array(
            values
                .iter()
                .filter_map(|value| toml_value_from_host_value(value, notification_value))
                .collect(),
        )),
        RetainedUiHostValue::Table(values) => Some(toml::Value::Table(
            values
                .iter()
                .filter_map(|(key, value)| {
                    Some((
                        key.clone(),
                        toml_value_from_host_value(value, notification_value)?,
                    ))
                })
                .collect(),
        )),
    }
}

#[cfg(test)]
thread_local! {
    static NOTIFICATION_TEXT_COPY_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_notification_text_copy(notification_value: bool) {
    if notification_value {
        NOTIFICATION_TEXT_COPY_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    }
}

#[cfg(not(test))]
fn record_notification_text_copy(_notification_value: bool) {}

#[cfg(test)]
pub(super) fn reset_notification_text_copy_count() {
    NOTIFICATION_TEXT_COPY_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(super) fn notification_text_copy_count() -> usize {
    NOTIFICATION_TEXT_COPY_COUNT.with(std::cell::Cell::get)
}
