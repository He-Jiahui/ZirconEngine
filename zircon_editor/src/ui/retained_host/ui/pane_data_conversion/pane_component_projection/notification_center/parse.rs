use toml::Value;

use super::attributes::{bool_value, first_string_value, normalized_tone, string_bool};
use super::entry::NotificationProjectionEntry;

pub(super) fn notification_entry_list(value: &Value) -> Vec<NotificationProjectionEntry> {
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
