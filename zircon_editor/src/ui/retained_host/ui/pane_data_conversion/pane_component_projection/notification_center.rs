use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use toml::Value;

const NOTIFICATIONS: &str = "notifications";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const FOCUSED_INDEX: &str = "focused_index";
const VISIBLE_LIMIT: &str = "visible_limit";
const EMPTY_TEXT: &str = "empty_text";

pub(super) fn projected_notification_center_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<String> {
    if !is_notification_center(component_role) {
        return None;
    }
    string_attribute(attributes, EMPTY_TEXT)
}

pub(super) fn projected_notification_center_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<String>> {
    is_notification_center(component_role).then(|| {
        projected_notification_entries(attributes)
            .into_iter()
            .map(|entry| entry.title)
            .collect()
    })
}

pub(super) fn projected_notification_center_structured_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<host_contract::TemplatePaneOptionData>> {
    if !is_notification_center(component_role) {
        return None;
    }

    let selected_id = string_attribute(attributes, SELECTED_NOTIFICATION_ID).unwrap_or_default();
    let focused_index = usize_attribute(attributes.get(FOCUSED_INDEX));
    Some(
        projected_notification_entries(attributes)
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                let selected = !selected_id.is_empty() && entry.matches_id(&selected_id);
                host_contract::TemplatePaneOptionData {
                    id: entry.id.into(),
                    label: entry.title.into(),
                    description: entry.message.into(),
                    tone: entry.tone.into(),
                    selected,
                    disabled: entry.disabled,
                    special: entry.unread,
                    unread: entry.unread,
                    focused: focused_index == Some(index),
                    ..host_contract::TemplatePaneOptionData::default()
                }
            })
            .collect(),
    )
}

fn is_notification_center(component_role: &str) -> bool {
    component_role == "notification-center"
}

#[derive(Clone, Debug)]
struct NotificationProjectionEntry {
    id: String,
    title: String,
    message: String,
    tone: String,
    unread: bool,
    disabled: bool,
}

impl NotificationProjectionEntry {
    fn new(id: String) -> Self {
        Self {
            title: id.clone(),
            id,
            message: String::new(),
            tone: "info".to_string(),
            unread: false,
            disabled: false,
        }
    }

    fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && (self.id == id || self.title == id)
    }
}

fn projected_notification_entries(
    attributes: &BTreeMap<String, Value>,
) -> Vec<NotificationProjectionEntry> {
    let visible_limit = attributes
        .get(VISIBLE_LIMIT)
        .and_then(|value| usize_attribute(Some(value)))
        .unwrap_or(usize::MAX);
    attributes
        .get(NOTIFICATIONS)
        .map(notification_entry_list)
        .unwrap_or_default()
        .into_iter()
        .take(visible_limit)
        .collect()
}

fn notification_entry_list(value: &Value) -> Vec<NotificationProjectionEntry> {
    match value {
        Value::Array(values) => values.iter().flat_map(notification_entry_list).collect(),
        Value::String(value) => notification_entry_from_string(value).into_iter().collect(),
        Value::Table(values) => notification_entry_from_table(values).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn notification_entry_from_string(value: &str) -> Option<NotificationProjectionEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = NotificationProjectionEntry::new(id);
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "title" | "label" | "text" | "name" => entry.title = value.to_string(),
            "message" | "body" | "description" | "detail" => entry.message = value.to_string(),
            "severity" | "level" | "kind" | "tone" => entry.tone = normalized_tone(value),
            "unread" | "new" => entry.unread = string_bool(value).unwrap_or(false),
            "disabled" => entry.disabled = string_bool(value).unwrap_or(false),
            "enabled" => entry.disabled = string_bool(value) == Some(false),
            _ => {}
        }
    }
    Some(entry)
}

fn notification_entry_from_table(
    values: &toml::map::Map<String, Value>,
) -> Option<NotificationProjectionEntry> {
    let id = first_string_value(
        values,
        &["id", "notification_id", "notificationId", "value", "key"],
    )?;
    if id.is_empty() {
        return None;
    }

    Some(NotificationProjectionEntry {
        title: first_string_value(values, &["title", "label", "text", "name"])
            .unwrap_or_else(|| id.clone()),
        message: first_string_value(values, &["message", "body", "description", "detail"])
            .unwrap_or_default(),
        tone: first_string_value(values, &["severity", "level", "kind", "tone"])
            .map(|value| normalized_tone(&value))
            .unwrap_or_else(|| "info".to_string()),
        unread: values
            .get("unread")
            .or_else(|| values.get("new"))
            .and_then(bool_value)
            .unwrap_or(false),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        id,
    })
}

fn first_string_value(values: &toml::map::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    attributes.get(key).and_then(string_value)
}

fn string_value(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bool_value(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => string_bool(value),
        _ => None,
    }
}

fn string_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn usize_attribute(value: Option<&Value>) -> Option<usize> {
    match value? {
        Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    }
}

fn normalized_tone(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "success" | "ok" | "done" => "success",
        "warning" | "warn" => "warning",
        "error" | "danger" | "failed" | "failure" => "error",
        _ => "info",
    }
    .to_string()
}
