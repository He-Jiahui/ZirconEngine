use std::collections::BTreeMap;

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
