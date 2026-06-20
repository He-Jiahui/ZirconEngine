use std::collections::BTreeMap;

use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::action_matches;

pub(super) fn demo_collection_input(action_id: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "group_toggled") => {
            Some(UiComponentShowcaseDemoEventInput::Toggle(false))
        }
        action if action_matches(action, "foldout_toggled") => {
            Some(UiComponentShowcaseDemoEventInput::Toggle(true))
        }
        action if action_matches(action, "inspector_section_toggled") => {
            Some(UiComponentShowcaseDemoEventInput::Toggle(false))
        }
        action if action_matches(action, "tree_row_toggled") => {
            Some(UiComponentShowcaseDemoEventInput::Toggle(false))
        }
        action if action_matches(action, "array_field_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(demo_array_field_value()),
        ),
        action if action_matches(action, "array_field_add_element") => {
            Some(UiComponentShowcaseDemoEventInput::AddElement {
                value: UiValue::String("MapField".to_string()),
            })
        }
        action if action_matches(action, "array_field_set_element") => {
            Some(UiComponentShowcaseDemoEventInput::SetElement {
                index: 1,
                value: UiValue::String("Vector3Field".to_string()),
            })
        }
        action if action_matches(action, "array_field_remove_element") => {
            Some(UiComponentShowcaseDemoEventInput::RemoveElement { index: 0 })
        }
        action if action_matches(action, "array_field_move_element") => {
            Some(UiComponentShowcaseDemoEventInput::MoveElement { from: 0, to: 1 })
        }
        action if action_matches(action, "map_field_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(demo_map_field_value()),
        ),
        action if action_matches(action, "map_field_add_entry") => {
            Some(UiComponentShowcaseDemoEventInput::AddMapEntry {
                key: "layer".to_string(),
                value: UiValue::String("Editor".to_string()),
            })
        }
        action if action_matches(action, "map_field_set_entry") => {
            Some(UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            })
        }
        action if action_matches(action, "map_field_remove_entry") => {
            Some(UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: "speed".to_string(),
            })
        }
        _ => None,
    }
}

pub(super) fn demo_array_field_value() -> UiValue {
    UiValue::Array(vec![
        UiValue::String("Label".to_string()),
        UiValue::String("Transform".to_string()),
        UiValue::String("Material".to_string()),
    ])
}

pub(super) fn demo_map_field_value() -> UiValue {
    let mut entries = BTreeMap::new();
    entries.insert("speed".to_string(), UiValue::Float(2.5));
    entries.insert("visible".to_string(), UiValue::Bool(false));
    UiValue::Map(entries)
}
