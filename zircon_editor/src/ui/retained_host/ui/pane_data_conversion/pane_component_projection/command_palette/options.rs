use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use toml::Value;

use super::attributes::{option_index, string_attribute_ref};
use super::entries::projected_command_entries;
use super::ids::command_id_set;

const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const QUERY: &str = "query";
const RECENT_COMMANDS: &str = "recent_commands";

pub(in crate::ui::retained_host::ui) fn projected_command_palette_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<String>> {
    projected_command_palette_option_rows(component_role, attributes).map(|(options, _)| options)
}

pub(in crate::ui::retained_host::ui) fn projected_command_palette_structured_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<host_contract::TemplatePaneOptionData>> {
    projected_command_palette_option_rows(component_role, attributes)
        .map(|(_, structured_options)| structured_options)
}

pub(in crate::ui::retained_host::ui) fn projected_command_palette_option_rows(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<(Vec<String>, Vec<host_contract::TemplatePaneOptionData>)> {
    if !is_command_palette(component_role) {
        return None;
    }

    let selected_id = string_attribute_ref(attributes, SELECTED_COMMAND_ID).unwrap_or_default();
    let recent_ids = command_id_set(attributes.get(RECENT_COMMANDS));
    let focused_index = option_index(attributes.get(FOCUSED_INDEX));
    let query = string_attribute_ref(attributes, QUERY);
    let entries = projected_command_entries(attributes);
    let options = entries.iter().map(|entry| entry.label.clone()).collect();

    let structured_options = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| host_contract::TemplatePaneOptionData {
            matched: entry.filter_matched || entry.matches_query(query),
            selected: !selected_id.is_empty() && entry.id == selected_id,
            disabled: entry.disabled,
            special: recent_ids.contains(&entry.id),
            focused: focused_index == Some(index),
            hovered: false,
            pressed: false,
            loading: false,
            id: entry.id.into(),
            label: entry.label.into(),
            description: entry.description.into(),
            ..host_contract::TemplatePaneOptionData::default()
        })
        .collect();
    Some((options, structured_options))
}

fn is_command_palette(component_role: &str) -> bool {
    component_role == "command-palette"
}
