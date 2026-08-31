use toml::Value;

use super::attributes::{
    bool_value, first_string_value, first_string_value_ref, normalized_tone, string_bool,
};
use super::entry::NotificationProjectionEntry;

#[cfg(test)]
thread_local! {
    static PARSE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn notification_entry_list_with_limit(
    value: &Value,
    limit: usize,
) -> Vec<NotificationProjectionEntry> {
    let mut entries = Vec::with_capacity(limit.min(64));
    push_notification_entries(value, limit, &mut entries);
    entries
}

fn push_notification_entries(
    value: &Value,
    limit: usize,
    entries: &mut Vec<NotificationProjectionEntry>,
) {
    if entries.len() >= limit {
        return;
    }

    match value {
        Value::Array(values) => {
            for value in values {
                push_notification_entries(value, limit, entries);
                if entries.len() >= limit {
                    break;
                }
            }
        }
        Value::String(value) => {
            record_parse_attempt();
            entries.extend(notification_entry_from_string(value));
        }
        Value::Table(values) => {
            record_parse_attempt();
            entries.extend(notification_entry_from_table(values));
        }
        _ => {}
    }
}

#[cfg(test)]
fn record_parse_attempt() {
    PARSE_COUNT.with(|count| count.set(count.get().saturating_add(1)));
}

#[cfg(not(test))]
fn record_parse_attempt() {}

#[cfg(test)]
pub(super) fn reset_parse_count() {
    PARSE_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(super) fn parse_count() -> usize {
    PARSE_COUNT.with(std::cell::Cell::get)
}

fn notification_entry_from_string(value: &str) -> Option<NotificationProjectionEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = NotificationProjectionEntry::new(id);
    let mut has_explicit_title = false;
    let mut has_explicit_tone = false;
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "title" | "label" | "text" | "name" => {
                entry.title = value.to_string();
                has_explicit_title = true;
            }
            "message" | "body" | "description" | "detail" => entry.message = value.to_string(),
            "severity" | "level" | "tone" => {
                entry.tone = normalized_tone(value).to_string();
                has_explicit_tone = true;
            }
            "kind" if !has_explicit_tone => {
                entry.tone = normalized_tone(value).to_string();
            }
            "unread" | "new" => entry.unread = string_bool(value).unwrap_or(false),
            "disabled" => entry.disabled = string_bool(value).unwrap_or(false),
            "enabled" => entry.disabled = string_bool(value) == Some(false),
            _ => {}
        }
    }
    if !has_explicit_title {
        entry.title = entry.id.clone();
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
        tone: first_string_value_ref(values, &["severity", "level", "tone", "kind"])
            .map(normalized_tone)
            .unwrap_or("info")
            .to_string(),
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
