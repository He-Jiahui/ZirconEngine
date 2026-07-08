use toml::Value;

use super::attributes::{bool_value, first_string_value};
use super::entry::CommandProjectionEntry;

pub(super) fn command_entry_list(value: &Value) -> Vec<CommandProjectionEntry> {
    match value {
        Value::Array(values) => values.iter().flat_map(command_entry_list).collect(),
        Value::String(value) => command_entry_from_string(value).into_iter().collect(),
        Value::Table(values) => command_entry_from_table(values).into_iter().collect(),
        _ => Vec::new(),
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
