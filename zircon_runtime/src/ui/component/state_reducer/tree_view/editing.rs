use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValidationState, UiValue,
    UiValueKind,
};

const EDITING_NODE_ID_PROPERTIES: [&str; 2] = ["editingNodeId", "editing_node_id"];
const EDITING_TEXT_PROPERTIES: [&str; 2] = ["editingText", "editing_text"];
const EDITING_INDEX_PROPERTIES: [&str; 2] = ["editingIndex", "editing_index"];
const RENAMED_NODE_ID_PROPERTIES: [&str; 2] = ["renamedNodeId", "renamed_node_id"];
const RENAMED_TEXT_PROPERTIES: [&str; 2] = ["renamedText", "renamed_text"];
const RENAME_COMMITTED_PROPERTIES: [&str; 2] = ["renameCommitted", "rename_committed"];

pub(in crate::ui::component::state_reducer) fn apply_begin_edit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<bool, UiComponentEventError> {
    if !super::is_tree_view(descriptor) {
        return Ok(false);
    }
    if !super::bool_setting(state, descriptor, "editable", true) {
        return Ok(true);
    }

    let Some((editing_index, node_id, editing_text)) = focused_edit_target(state, descriptor)
    else {
        return Ok(true);
    };

    super::super::set_value(state, "editing".to_string(), UiValue::Bool(true));
    super::super::set_value(
        state,
        editing_node_id_property(state, descriptor).to_string(),
        UiValue::String(node_id),
    );
    super::super::set_value(
        state,
        editing_text_property(state, descriptor).to_string(),
        UiValue::String(editing_text),
    );
    super::super::set_value(
        state,
        editing_index_property(state, descriptor).to_string(),
        UiValue::Int(editing_index as i64),
    );
    super::super::set_value(
        state,
        rename_committed_property(state, descriptor).to_string(),
        UiValue::Bool(false),
    );
    super::super::set_value(
        state,
        renamed_node_id_property(state, descriptor).to_string(),
        UiValue::String(String::new()),
    );
    super::super::set_value(
        state,
        renamed_text_property(state, descriptor).to_string(),
        UiValue::String(String::new()),
    );
    state.flags.focused = true;
    Ok(true)
}

pub(in crate::ui::component::state_reducer) fn apply_cancel_editing(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<bool, UiComponentEventError> {
    if !super::is_tree_view(descriptor) || !tree_is_editing(state, descriptor) {
        return Ok(false);
    }

    clear_editing_state(state, descriptor);
    super::super::set_value(
        state,
        rename_committed_property(state, descriptor).to_string(),
        UiValue::Bool(false),
    );
    Ok(true)
}

pub(in crate::ui::component::state_reducer) fn apply_commit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Result<bool, UiComponentEventError> {
    if !super::is_tree_view(descriptor) || !is_editing_text_property(property) {
        return Ok(false);
    }
    if !tree_is_editing(state, descriptor) {
        return Ok(false);
    }

    let Some(node_id) = editing_node_id(state, descriptor) else {
        return Ok(false);
    };
    let Some(rename_text) = super::string_value(value).map(str::to_owned) else {
        state.validation = UiValidationState::error(format!(
            "tree rename commit requires string text for `{property}`"
        ));
        return Err(UiComponentEventError::InvalidValueKind {
            property: property.to_string(),
            expected: UiValueKind::String,
            actual: value.kind(),
        });
    };

    super::super::set_value(
        state,
        editing_text_property(state, descriptor).to_string(),
        UiValue::String(rename_text.clone()),
    );
    super::super::set_value(
        state,
        renamed_node_id_property(state, descriptor).to_string(),
        UiValue::String(node_id),
    );
    super::super::set_value(
        state,
        renamed_text_property(state, descriptor).to_string(),
        UiValue::String(rename_text),
    );
    super::super::set_value(
        state,
        rename_committed_property(state, descriptor).to_string(),
        UiValue::Bool(true),
    );
    clear_editing_state(state, descriptor);
    Ok(true)
}

fn focused_edit_target(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Option<(usize, String, String)> {
    let node_ids = super::ordered_node_ids(state, descriptor);
    if node_ids.is_empty() {
        return None;
    }

    let index = super::current_tree_index(state, descriptor, &node_ids);
    let node_id = node_ids.into_iter().nth(index)?;
    let editing_text = tree_node_label(state, descriptor, &node_id)
        .unwrap_or(node_id)
        .to_string();
    Some((index, node_id.to_string(), editing_text))
}

fn clear_editing_state(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    super::super::set_value(state, "editing".to_string(), UiValue::Bool(false));
    super::super::set_value(
        state,
        editing_node_id_property(state, descriptor).to_string(),
        UiValue::String(String::new()),
    );
    super::super::set_value(
        state,
        editing_text_property(state, descriptor).to_string(),
        UiValue::String(String::new()),
    );
    super::super::set_value(
        state,
        editing_index_property(state, descriptor).to_string(),
        UiValue::Int(-1),
    );
}

fn tree_is_editing(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    super::bool_setting(state, descriptor, "editing", false)
        || editing_node_id(state, descriptor).is_some()
}

fn editing_node_id(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Option<String> {
    EDITING_NODE_ID_PROPERTIES
        .iter()
        .filter_map(|property| {
            state.values.get(*property).or_else(|| {
                descriptor
                    .prop(property)
                    .and_then(|schema| schema.default_value.as_ref())
            })
        })
        .filter_map(super::string_value)
        .find(|node_id| !node_id.is_empty())
        .map(str::to_owned)
}

fn tree_node_label<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    target_id: &str,
) -> Option<&'a str> {
    for property in super::NODE_PROPERTIES {
        if let Some(value) = state.values.get(property).or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
        }) {
            if let Some(label) = find_tree_node_label(value, target_id) {
                return Some(label);
            }
        }
    }
    None
}

fn find_tree_node_label<'a>(value: &'a UiValue, target_id: &str) -> Option<&'a str> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .find_map(|value| find_tree_node_label(value, target_id)),
        UiValue::String(value) | UiValue::Enum(value) => {
            (value == target_id).then_some(value.as_str())
        }
        UiValue::Map(values) => {
            if tree_node_identity(values) == Some(target_id) {
                return tree_node_display_text(values);
            }
            for property in ["children", "nodes", "items", "options"] {
                if let Some(value) = values.get(property) {
                    if let Some(label) = find_tree_node_label(value, target_id) {
                        return Some(label);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn tree_node_identity(values: &BTreeMap<String, UiValue>) -> Option<&str> {
    for property in ["id", "value", "row_id", "rowId", "node_id", "nodeId", "key"] {
        if let Some(value) = values.get(property).and_then(borrowed_string_value) {
            return Some(value);
        }
    }
    None
}

fn tree_node_display_text(values: &BTreeMap<String, UiValue>) -> Option<&str> {
    for property in ["label", "text", "name", "title", "id", "value"] {
        if let Some(value) = values.get(property).and_then(borrowed_string_value) {
            return Some(value);
        }
    }
    None
}

fn borrowed_string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

fn editing_node_id_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &EDITING_NODE_ID_PROPERTIES)
}

fn editing_text_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &EDITING_TEXT_PROPERTIES)
}

fn editing_index_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &EDITING_INDEX_PROPERTIES)
}

fn renamed_node_id_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &RENAMED_NODE_ID_PROPERTIES)
}

fn renamed_text_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &RENAMED_TEXT_PROPERTIES)
}

fn rename_committed_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    preferred_property(state, descriptor, &RENAME_COMMITTED_PROPERTIES)
}

fn preferred_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    properties: &'static [&'static str],
) -> &'static str {
    properties
        .iter()
        .copied()
        .find(|property| state.values.contains_key(*property))
        .or_else(|| {
            properties
                .iter()
                .copied()
                .find(|property| descriptor.prop(property).is_some())
        })
        .unwrap_or(properties[0])
}

fn is_editing_text_property(property: &str) -> bool {
    EDITING_TEXT_PROPERTIES.contains(&property)
}

#[cfg(test)]
#[path = "editing/borrowed_node_lookup_tests.rs"]
mod borrowed_node_lookup_tests;
