use std::collections::{BTreeMap, BTreeSet};

use crate::ui::retained_host as host_contract;
use toml::Value;

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";
const DISABLED_COMMANDS: &str = "disabled_commands";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const QUERY: &str = "query";
const RECENT_COMMANDS: &str = "recent_commands";

pub(in crate::ui::retained_host::ui) fn projected_command_palette_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<String>> {
    is_command_palette(component_role).then(|| {
        projected_command_entries(attributes)
            .into_iter()
            .map(|entry| entry.label)
            .collect()
    })
}

pub(in crate::ui::retained_host::ui) fn projected_command_palette_structured_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<host_contract::TemplatePaneOptionData>> {
    if !is_command_palette(component_role) {
        return None;
    }

    let selected_id = string_attribute(attributes, SELECTED_COMMAND_ID).unwrap_or_default();
    let disabled_ids = command_id_set(attributes.get(DISABLED_COMMANDS));
    let recent_ids = command_id_set(attributes.get(RECENT_COMMANDS));
    let focused_index = option_index(attributes.get(FOCUSED_INDEX));
    let query = string_attribute(attributes, QUERY)
        .map(|query| query.trim().to_ascii_lowercase())
        .filter(|query| !query.is_empty());

    Some(
        projected_command_entries(attributes)
            .into_iter()
            .enumerate()
            .map(|(index, entry)| host_contract::TemplatePaneOptionData {
                matched: entry.matches_query(query.as_deref()),
                selected: !selected_id.is_empty() && entry.id == selected_id,
                disabled: entry.disabled || disabled_ids.contains(&entry.id),
                special: recent_ids.contains(&entry.id),
                focused: focused_index == Some(index),
                hovered: false,
                pressed: false,
                loading: false,
                id: entry.id.into(),
                label: entry.label.into(),
                ..host_contract::TemplatePaneOptionData::default()
            })
            .collect(),
    )
}

fn is_command_palette(component_role: &str) -> bool {
    component_role == "command-palette"
}

#[derive(Clone, Debug)]
struct CommandProjectionEntry {
    id: String,
    label: String,
    disabled: bool,
}

impl CommandProjectionEntry {
    fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            disabled: false,
        }
    }

    fn matches_query(&self, query: Option<&str>) -> bool {
        let Some(query) = query else {
            return false;
        };
        self.id.to_ascii_lowercase().contains(query)
            || self.label.to_ascii_lowercase().contains(query)
    }
}

fn projected_command_entries(attributes: &BTreeMap<String, Value>) -> Vec<CommandProjectionEntry> {
    let commands = attributes
        .get(COMMANDS)
        .map(command_entry_list)
        .unwrap_or_default();
    let Some(filtered) = attributes.get(FILTERED_COMMANDS) else {
        return commands;
    };

    command_id_values(filtered)
        .into_iter()
        .filter_map(|id| {
            commands
                .iter()
                .find(|entry| entry.id == id)
                .cloned()
                .or_else(|| (!id.is_empty()).then(|| CommandProjectionEntry::new(id)))
        })
        .collect()
}

fn command_entry_list(value: &Value) -> Vec<CommandProjectionEntry> {
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
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        id,
    })
}

fn command_id_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .map(command_id_values)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn command_id_values(value: &Value) -> Vec<String> {
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
        Value::String(value) => match value.trim() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn option_index(value: Option<&Value>) -> Option<usize> {
    value
        .and_then(Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
}
