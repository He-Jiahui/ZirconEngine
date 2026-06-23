use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentKeyboardAction, UiComponentState,
    UiValidationState, UiValue,
};

mod editing;

pub(super) use editing::{apply_begin_edit, apply_cancel_editing, apply_commit};

const NODE_PROPERTIES: [&str; 3] = ["nodes", "items", "options"];
const EXPANDED_CONTROL_PROPERTIES: [&str; 2] = ["expanded_items", "expandedItems"];
const DEFAULT_EXPANDED_PROPERTIES: [&str; 2] = ["default_expanded_items", "defaultExpandedItems"];
const SELECTED_CONTROL_PROPERTIES: [&str; 2] = ["selected_items", "selectedItems"];

pub(super) fn is_tree_view(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "tree-view" | "folder-tree" | "mui-x-tree-view"
    ) || matches!(
        descriptor.id.as_str(),
        "TreeView" | "MaterialTreeView" | "FolderTree"
    )
}

pub(super) fn apply_keyboard_expand_collapse(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<bool, UiComponentEventError> {
    if !is_tree_view(descriptor) {
        return Ok(false);
    }

    match action {
        UiComponentKeyboardAction::Increment => {
            Ok(set_focused_node_expanded(state, descriptor, true))
        }
        UiComponentKeyboardAction::Decrement => {
            Ok(set_focused_node_expanded(state, descriptor, false))
        }
        _ => Ok(false),
    }
}

pub(super) fn apply_toggle_expanded(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    expanded: bool,
) -> Result<bool, UiComponentEventError> {
    if !is_tree_view(descriptor) {
        return Ok(false);
    }
    Ok(set_focused_node_expanded(state, descriptor, expanded))
}

pub(super) fn apply_select_option(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    option_id: &str,
    selected: bool,
) -> Result<bool, UiComponentEventError> {
    if !is_tree_view(descriptor) {
        return Ok(false);
    }

    let node_ids = ordered_node_ids(state, descriptor);
    let Some(target_index) = node_ids.iter().position(|node_id| node_id == option_id) else {
        return Ok(false);
    };

    if option_is_disabled(state, option_id) {
        state.validation = UiValidationState::error(format!(
            "disabled tree node `{option_id}` cannot be selected"
        ));
        return Err(UiComponentEventError::DisabledOption {
            component_id: descriptor.id.clone(),
            option_id: option_id.to_string(),
        });
    }

    let multi_select = tree_multi_select(state, descriptor, property);
    let range_selecting = selected && tree_range_selecting(state, descriptor);
    if multi_select || range_selecting || is_selected_control_property(property) {
        apply_multi_select_option(
            state,
            descriptor,
            property,
            option_id,
            selected,
            target_index,
            &node_ids,
            range_selecting,
        );
    } else {
        apply_single_select_option(state, property, option_id, selected, target_index);
    }
    Ok(true)
}

fn set_focused_node_expanded(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    expanded: bool,
) -> bool {
    let Some(node_id) = focused_node_id(state, descriptor) else {
        return false;
    };

    let mut expanded_ids = expanded_node_ids(state, descriptor);
    if expanded {
        push_unique(&mut expanded_ids, node_id);
    } else {
        expanded_ids.retain(|id| id != &node_id);
    }

    let has_expanded_nodes = !expanded_ids.is_empty();
    let value = UiValue::Array(
        expanded_ids
            .into_iter()
            .map(UiValue::String)
            .collect::<Vec<_>>(),
    );
    super::set_value(
        state,
        expanded_control_property(state, descriptor).to_string(),
        value,
    );
    super::set_value(
        state,
        "expanded".to_string(),
        UiValue::Bool(has_expanded_nodes),
    );
    state.flags.expanded = has_expanded_nodes;
    true
}

fn apply_multi_select_option(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    option_id: &str,
    selected: bool,
    target_index: usize,
    node_ids: &[String],
    range_selecting: bool,
) {
    let property = selected_control_property(state, descriptor, property);
    let selected_ids = if range_selecting {
        range_selected_node_ids(state, descriptor, node_ids, target_index)
    } else {
        let mut selected_ids = selected_node_ids(state, descriptor, property);
        if selected {
            push_unique(&mut selected_ids, option_id.to_string());
        } else {
            selected_ids.retain(|id| id != option_id);
        }
        selected_ids
    };

    let has_selection = !selected_ids.is_empty();
    super::set_value(
        state,
        property.to_string(),
        UiValue::Array(
            selected_ids
                .into_iter()
                .map(UiValue::String)
                .collect::<Vec<_>>(),
        ),
    );
    set_tree_selection_focus(state, target_index);
    if !range_selecting {
        super::set_value(
            state,
            "selection_anchor_index".to_string(),
            UiValue::Int(target_index as i64),
        );
    }
    state.flags.selected = has_selection;
}

fn apply_single_select_option(
    state: &mut UiComponentState,
    property: &str,
    option_id: &str,
    selected: bool,
    target_index: usize,
) {
    let property = if property.is_empty() {
        "value"
    } else {
        property
    };
    let value = if selected {
        UiValue::String(option_id.to_string())
    } else {
        UiValue::Null
    };
    super::set_value(state, property.to_string(), value);
    set_tree_selection_focus(state, target_index);
    super::set_value(
        state,
        "selection_anchor_index".to_string(),
        UiValue::Int(target_index as i64),
    );
    state.flags.selected = selected;
}

fn set_tree_selection_focus(state: &mut UiComponentState, target_index: usize) {
    let index = UiValue::Int(target_index as i64);
    super::set_value(state, "focused_index".to_string(), index.clone());
    super::set_value(state, "selected_index".to_string(), index);
    state.flags.focused = true;
}

fn range_selected_node_ids(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    node_ids: &[String],
    target_index: usize,
) -> Vec<String> {
    let anchor = int_setting(state, descriptor, "selection_anchor_index")
        .or_else(|| int_setting(state, descriptor, "selectionAnchorIndex"))
        .unwrap_or(target_index as i64)
        .clamp(0, (node_ids.len() - 1) as i64) as usize;
    let start = anchor.min(target_index);
    let end = anchor.max(target_index);
    let mut selected_ids = Vec::new();
    for node_id in &node_ids[start..=end] {
        if !option_is_disabled(state, node_id) {
            push_unique(&mut selected_ids, node_id.clone());
        }
    }
    selected_ids
}

fn selected_control_property<'a>(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    requested_property: &'a str,
) -> &'a str {
    if is_selected_control_property(requested_property) {
        return requested_property;
    }
    for property in SELECTED_CONTROL_PROPERTIES {
        if state.values.contains_key(property) || descriptor.prop(property).is_some() {
            return property;
        }
    }
    "selected_items"
}

fn selected_node_ids(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(value) = state.values.get(property).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
    }) {
        collect_string_ids(value, &mut ids);
    }
    ids
}

fn expanded_control_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    for property in EXPANDED_CONTROL_PROPERTIES {
        if state.values.contains_key(property) || descriptor.prop(property).is_some() {
            return property;
        }
    }
    if state.values.contains_key("defaultExpandedItems")
        || descriptor.prop("defaultExpandedItems").is_some()
    {
        return "expandedItems";
    }
    "expanded_items"
}

fn expanded_node_ids(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Vec<String> {
    for property in EXPANDED_CONTROL_PROPERTIES {
        if let Some(value) = state.values.get(property) {
            let mut ids = Vec::new();
            collect_string_ids(value, &mut ids);
            return ids;
        }
    }

    let mut ids = Vec::new();
    for property in DEFAULT_EXPANDED_PROPERTIES {
        if let Some(value) = state.values.get(property).or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
        }) {
            collect_string_ids(value, &mut ids);
        }
    }
    ids
}

fn focused_node_id(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Option<String> {
    let node_ids = ordered_node_ids(state, descriptor);
    if node_ids.is_empty() {
        return None;
    }

    let index = current_tree_index(state, descriptor, &node_ids);
    node_ids.get(index).cloned()
}

fn ordered_node_ids(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Vec<String> {
    let mut node_ids = Vec::new();
    for property in NODE_PROPERTIES {
        if let Some(value) = state.values.get(property).or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
        }) {
            collect_tree_node_ids(value, &mut node_ids);
        }
    }
    node_ids
}

fn current_tree_index(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    node_ids: &[String],
) -> usize {
    int_setting(state, descriptor, "focused_index")
        .or_else(|| int_setting(state, descriptor, "selected_index"))
        .or_else(|| current_value_index(state, node_ids))
        .unwrap_or(0)
        .clamp(0, (node_ids.len() - 1) as i64) as usize
}

fn current_value_index(state: &UiComponentState, node_ids: &[String]) -> Option<i64> {
    ["value", "value_text", "group_value"]
        .into_iter()
        .filter_map(|property| state.values.get(property).and_then(string_value))
        .find_map(|value| {
            node_ids
                .iter()
                .position(|node_id| node_id == &value)
                .map(|index| index as i64)
        })
}

fn collect_tree_node_ids(value: &UiValue, out: &mut Vec<String>) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_tree_node_ids(value, out);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => push_unique(out, value.clone()),
        UiValue::Map(values) => {
            if let Some(value) = values
                .get("id")
                .or_else(|| values.get("value"))
                .or_else(|| values.get("row_id"))
                .or_else(|| values.get("rowId"))
                .or_else(|| values.get("node_id"))
                .or_else(|| values.get("nodeId"))
                .or_else(|| values.get("key"))
            {
                collect_string_ids(value, out);
            }
            for property in ["children", "nodes", "items", "options"] {
                if let Some(value) = values.get(property) {
                    collect_tree_node_ids(value, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_string_ids(value: &UiValue, out: &mut Vec<String>) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_string_ids(value, out);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => push_unique(out, value.clone()),
        UiValue::Flags(values) => {
            for value in values {
                push_unique(out, value.clone());
            }
        }
        UiValue::Map(values) => {
            if let Some(value) = values.get("id").or_else(|| values.get("value")) {
                collect_string_ids(value, out);
            }
        }
        _ => {}
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn int_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<i64> {
    state.values.get(property).and_then(int_value).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
            .and_then(int_value)
    })
}

fn int_value(value: &UiValue) -> Option<i64> {
    match value {
        UiValue::Int(value) => Some(*value),
        _ => None,
    }
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}

fn tree_multi_select(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    requested_property: &str,
) -> bool {
    is_selected_control_property(requested_property)
        || bool_setting(state, descriptor, "multi_select", false)
        || bool_setting(state, descriptor, "multiSelect", false)
        || bool_setting(state, descriptor, "checkboxSelection", false)
}

fn tree_range_selecting(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    bool_setting(state, descriptor, "range_selecting", false)
        || bool_setting(state, descriptor, "rangeSelecting", false)
        || bool_setting(state, descriptor, "shift_selecting", false)
        || bool_setting(state, descriptor, "shiftSelecting", false)
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    state
        .values
        .get(property)
        .and_then(bool_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(bool_value)
        })
        .unwrap_or(default_value)
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn option_is_disabled(state: &UiComponentState, option_id: &str) -> bool {
    state
        .values
        .get("disabled_options")
        .is_some_and(|value| option_id_list_contains(value, option_id))
}

fn option_id_list_contains(value: &UiValue, option_id: &str) -> bool {
    match value {
        UiValue::Array(values) => values
            .iter()
            .any(|value| option_id_list_contains(value, option_id)),
        UiValue::String(value) | UiValue::Enum(value) => value == option_id,
        UiValue::Flags(values) => values.iter().any(|value| value == option_id),
        _ => false,
    }
}

fn is_selected_control_property(property: &str) -> bool {
    SELECTED_CONTROL_PROPERTIES
        .iter()
        .any(|selected_property| property == *selected_property)
}
