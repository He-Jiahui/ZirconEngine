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

    command_id_values(filtered)
        .into_iter()
        .filter_map(|id| {
            command_index
                .get(id.as_str())
                .map(|index| commands[*index].clone())
                .or_else(|| (!id.is_empty()).then(|| CommandProjectionEntry::new(id)))
                .map(CommandProjectionEntry::with_filter_matched)
        })
        .collect()
}
