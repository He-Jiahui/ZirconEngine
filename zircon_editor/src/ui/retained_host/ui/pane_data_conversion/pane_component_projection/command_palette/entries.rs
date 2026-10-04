use std::collections::{BTreeMap, HashMap};

use toml::Value;

use super::entry::CommandProjectionEntry;
use super::ids::command_id_values;
use super::parse::command_entry_list;

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";

pub(super) fn projected_command_entries(
    attributes: &BTreeMap<String, Value>,
) -> Vec<CommandProjectionEntry> {
    let commands = attributes
        .get(COMMANDS)
        .map(command_entry_list)
        .unwrap_or_default();
    let Some(filtered) = attributes.get(FILTERED_COMMANDS) else {
        return commands;
    };
    let mut command_index = HashMap::with_capacity(commands.len());
    for (index, entry) in commands.iter().enumerate() {
        command_index.entry(entry.id.as_str()).or_insert(index);
    }

    let ids = command_id_values(filtered);
    let mut entries = Vec::with_capacity(ids.len());
    for id in ids {
        let entry = if let Some(index) = command_index.get(id.as_str()) {
            Some(commands[*index].clone())
        } else if id.is_empty() {
            None
        } else {
            Some(CommandProjectionEntry::new(id))
        };
        if let Some(entry) = entry {
            entries.push(entry.with_filter_matched());
        }
    }
    entries
}

#[cfg(test)]
#[path = "tests/entries_optimization_batch_20260830ce_editor_tests.rs"]
mod optimization_batch_20260830ce_editor_tests;
