use toml::Value;

use super::attributes::{bool_value, first_string_value};
use super::entry::CommandProjectionEntry;

pub(super) fn command_entry_list(value: &Value) -> Vec<CommandProjectionEntry> {
    let mut entries = Vec::new();
    append_command_entries(value, &mut entries);
    entries
}

fn append_command_entries(value: &Value, entries: &mut Vec<CommandProjectionEntry>) {
    match value {
        Value::Array(values) => {
            for value in values {
                append_command_entries(value, entries);
            }
        }
        Value::String(value) => entries.extend(command_entry_from_string(value)),
        Value::Table(values) => entries.extend(command_entry_from_table(values)),
        _ => {}
    }
}

fn command_entry_from_string(value: &str) -> Option<CommandProjectionEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = CommandProjectionEntry::new(id);
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "label" | "text" | "title" | "name" => entry.label = value.to_string(),
            "description" | "subtitle" | "hint" | "shortcut" | "keybinding" | "keys"
            | "accelerator" => entry.description = value.to_string(),
            "disabled" => entry.disabled = matches!(value, "true" | "1" | "yes"),
            _ => {}
        }
    }
    Some(entry)
}

fn command_entry_from_table(
    values: &toml::map::Map<String, Value>,
) -> Option<CommandProjectionEntry> {
    let id = first_string_value(values, &["id", "command_id", "commandId", "value", "key"])?;
    if id.is_empty() {
        return None;
    }

    Some(CommandProjectionEntry {
        label: first_string_value(values, &["label", "text", "title", "name", "value_text"])
            .unwrap_or_else(|| id.clone()),
        description: first_string_value(
            values,
            &[
                "description",
                "subtitle",
                "hint",
                "shortcut",
                "keybinding",
                "keys",
                "accelerator",
            ],
        )
        .unwrap_or_default(),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        filter_matched: false,
        id,
    })
}

#[cfg(test)]
#[path = "parse/direct_append_tests.rs"]
mod direct_append_tests;
