use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::super::action_matches;
use super::values::parse_collection_edit_value;

pub(super) fn demo_collection_edit_input(
    action_id: &str,
    value: &str,
) -> Option<UiComponentShowcaseDemoEventInput> {
    if action_matches(action_id, "array_field_remove_element") {
        return value
            .strip_prefix("array-")
            .and_then(|index| index.parse::<usize>().ok())
            .map(|index| UiComponentShowcaseDemoEventInput::RemoveElement { index });
    }
    if action_matches(action_id, "array_field_move_element") {
        return parse_array_move(value);
    }
    if action_matches(action_id, "array_field_set_element") {
        return parse_array_set(value);
    }
    if action_matches(action_id, "map_field_remove_entry") {
        return value.strip_prefix("map-").map(|key| {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: key.to_string(),
            }
        });
    }
    if action_matches(action_id, "map_field_set_entry") {
        return parse_map_set(value);
    }
    None
}

fn parse_array_move(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (row_id, to) = value.split_once('=')?;
    let from = row_id
        .strip_prefix("array-")
        .and_then(|index| index.parse::<usize>().ok())?;
    let to = to.parse::<usize>().ok()?;
    Some(UiComponentShowcaseDemoEventInput::MoveElement { from, to })
}

fn parse_array_set(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (row_id, value) = value.split_once('=')?;
    let index = row_id
        .strip_prefix("array-")
        .and_then(|index| index.parse::<usize>().ok())?;
    Some(UiComponentShowcaseDemoEventInput::SetElement {
        index,
        value: parse_collection_edit_value(value),
    })
}

fn parse_map_set(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (row_id, value) = value.split_once('=')?;
    if let Some(key) = row_id.strip_prefix("key:map-") {
        return Some(UiComponentShowcaseDemoEventInput::RenameMapEntry {
            from_key: key.to_string(),
            to_key: value.to_string(),
        });
    }
    row_id
        .strip_prefix("map-")
        .map(|key| UiComponentShowcaseDemoEventInput::SetMapEntry {
            key: key.to_string(),
            value: parse_collection_edit_value(value),
        })
}
