use std::collections::HashSet;

use zircon_runtime_interface::ui::{component::UiValue, tree::UiTemplateNodeMetadata};

pub(super) const NODE_PROPERTIES: [&str; 3] = ["nodes", "items", "options"];
pub(super) const SELECTED_PROPERTIES: [&str; 2] = ["selected_items", "selectedItems"];
pub(super) const ANCHOR_PROPERTIES: [&str; 2] = ["selection_anchor_index", "selectionAnchorIndex"];
const EXPANDED_PROPERTIES: [&str; 2] = ["expanded_items", "expandedItems"];
const DEFAULT_EXPANDED_PROPERTIES: [&str; 2] = ["default_expanded_items", "defaultExpandedItems"];
pub(super) const OPTION_ID_PROPERTIES: [&str; 8] = [
    "itemId",
    "item_id",
    "nodeId",
    "node_id",
    "optionId",
    "option_id",
    "id",
    "value",
];

const TREE_OWNER_ROLES: [&str; 3] = ["tree-view", "mui-x-tree-view", "folder-tree"];
const TREE_ITEM_ROLES: [&str; 2] = ["tree-item", "tree-row"];
const TREE_NODE_IDENTITY_PROPERTIES: [&str; 13] = [
    "itemId",
    "item_id",
    "nodeId",
    "node_id",
    "optionId",
    "option_id",
    "id",
    "value",
    "row_id",
    "rowId",
    "key",
    "name",
    "title",
];
const TREE_NODE_LABEL_PROPERTIES: [&str; 6] = ["label", "text", "name", "title", "id", "value"];

pub(super) fn is_default_tree_view_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    is_tree_view_owner(metadata) || role_is_one_of(metadata, &TREE_ITEM_ROLES)
}

pub(super) fn is_tree_view_owner(metadata: &UiTemplateNodeMetadata) -> bool {
    role_is_one_of(metadata, &TREE_OWNER_ROLES)
}

pub(super) fn tree_item_id(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    OPTION_ID_PROPERTIES
        .iter()
        .find_map(|property| string_attribute(metadata, property))
        .filter(|id| !id.is_empty())
}

pub(super) fn tree_node_ids<'a>(metadata: &'a UiTemplateNodeMetadata) -> Vec<&'a str> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for property in NODE_PROPERTIES {
        if let Some(value) = metadata.attributes.get(property) {
            collect_tree_node_ids(value, &mut ids, &mut seen);
        }
    }
    ids
}

pub(super) fn tree_nodes_property(metadata: &UiTemplateNodeMetadata) -> Option<&'static str> {
    NODE_PROPERTIES.iter().copied().find(|property| {
        matches!(
            metadata.attributes.get(*property),
            Some(toml::Value::Array(_))
        )
    })
}

pub(super) fn tree_node_ids_for_property<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    property: &str,
) -> Vec<&'a str> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    if let Some(value) = metadata.attributes.get(property) {
        collect_tree_node_ids(value, &mut ids, &mut seen);
    }
    ids
}

pub(super) fn tree_node_values_for_property(
    metadata: &UiTemplateNodeMetadata,
    property: &str,
) -> Vec<UiValue> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_array)
        .map(|values| values.iter().map(UiValue::from_toml).collect())
        .unwrap_or_default()
}

pub(super) fn selected_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    if metadata.attributes.contains_key("selectedItems")
        || super::semantics::component_role_is(metadata, "mui-x-tree-view")
    {
        return "selectedItems";
    }
    for property in SELECTED_PROPERTIES {
        if metadata.attributes.contains_key(property) {
            return property;
        }
    }
    "selected_items"
}

pub(super) fn selected_ids(metadata: &UiTemplateNodeMetadata, property: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    if let Some(value) = metadata.attributes.get(property) {
        collect_owned_string_ids(value, &mut ids, &mut seen);
    }
    ids
}

pub(super) fn range_selected_ids(
    metadata: &UiTemplateNodeMetadata,
    node_ids: &[&str],
    target_index: usize,
) -> Vec<String> {
    let anchor = anchor_index(metadata)
        .unwrap_or(target_index as i64)
        .clamp(0, (node_ids.len() - 1) as i64) as usize;
    let start = anchor.min(target_index);
    let end = anchor.max(target_index);
    let disabled_ids = disabled_option_ids(metadata);
    let mut selected = Vec::with_capacity(end - start + 1);
    for node_id in &node_ids[start..=end] {
        if !disabled_ids.contains(*node_id) {
            selected.push((*node_id).to_string());
        }
    }
    selected
}

pub(super) fn toggled_selected_ids(
    mut selected_ids: Vec<String>,
    option_id: &str,
    selected: bool,
) -> Vec<String> {
    if selected {
        push_unique(&mut selected_ids, option_id.to_string());
    } else {
        selected_ids.retain(|id| id != option_id);
    }
    selected_ids
}

pub(super) fn tree_multi_select(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "multi_select")
        || bool_attribute(metadata, "multiSelect")
        || bool_attribute(metadata, "checkboxSelection")
}

pub(super) fn tree_reorderable(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "reorderable")
        || bool_attribute(metadata, "itemsReordering")
        || bool_attribute(metadata, "items_reordering")
}

pub(super) fn tree_option_is_disabled(metadata: &UiTemplateNodeMetadata, option_id: &str) -> bool {
    metadata
        .attributes
        .get("disabled_options")
        .or_else(|| metadata.attributes.get("disabledItems"))
        .or_else(|| metadata.attributes.get("disabled_items"))
        .is_some_and(|value| value_contains_string(value, option_id))
}

pub(super) fn tree_editable(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata
        .attributes
        .get("editable")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
}

pub(super) fn anchor_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    if metadata.attributes.contains_key("selectionAnchorIndex")
        || super::semantics::component_role_is(metadata, "mui-x-tree-view")
    {
        return "selectionAnchorIndex";
    }
    for property in ANCHOR_PROPERTIES {
        if metadata.attributes.contains_key(property) {
            return property;
        }
    }
    "selection_anchor_index"
}

pub(super) fn anchor_index(metadata: &UiTemplateNodeMetadata) -> Option<i64> {
    ANCHOR_PROPERTIES
        .iter()
        .find_map(|property| int_attribute(metadata, property))
}

pub(super) fn expanded_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    if super::semantics::component_role_is(metadata, "mui-x-tree-view")
        || metadata.attributes.contains_key("expandedItems")
        || metadata.attributes.contains_key("defaultExpandedItems")
    {
        return "expandedItems";
    }
    for property in EXPANDED_PROPERTIES {
        if metadata.attributes.contains_key(property) {
            return property;
        }
    }
    "expanded_items"
}

pub(super) fn expanded_ids(metadata: &UiTemplateNodeMetadata, property: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    if let Some(value) = metadata.attributes.get(property) {
        collect_owned_string_ids(value, &mut ids, &mut seen);
        return ids;
    }
    for property in DEFAULT_EXPANDED_PROPERTIES {
        if let Some(value) = metadata.attributes.get(property) {
            collect_owned_string_ids(value, &mut ids, &mut seen);
        }
    }
    ids
}

pub(super) fn editing_node_id_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "editing_node_id", "editingNodeId")
}

pub(super) fn editing_text_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "editing_text", "editingText")
}

pub(super) fn editing_index_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "editing_index", "editingIndex")
}

pub(super) fn rename_committed_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "rename_committed", "renameCommitted")
}

pub(super) fn renamed_node_id_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "renamed_node_id", "renamedNodeId")
}

pub(super) fn renamed_text_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    preferred_tree_alias_property(metadata, "renamed_text", "renamedText")
}

pub(super) fn tree_node_label(
    metadata: &UiTemplateNodeMetadata,
    target_id: &str,
) -> Option<String> {
    for property in NODE_PROPERTIES {
        if let Some(value) = metadata.attributes.get(property) {
            if let Some(label) = find_tree_node_label(value, target_id) {
                return Some(label);
            }
        }
    }
    None
}

pub(super) fn string_array_value(values: &[String]) -> UiValue {
    UiValue::Array(values.iter().cloned().map(UiValue::String).collect())
}

fn collect_tree_node_ids<'a>(
    value: &'a toml::Value,
    out: &mut Vec<&'a str>,
    seen: &mut HashSet<&'a str>,
) {
    match value {
        toml::Value::Array(values) => {
            for value in values {
                collect_tree_node_ids(value, out, seen);
            }
        }
        toml::Value::String(value) => push_unique_borrowed(out, seen, value),
        toml::Value::Table(values) => {
            for property in OPTION_ID_PROPERTIES {
                if let Some(value) = values.get(property) {
                    collect_borrowed_string_ids(value, out, seen);
                    break;
                }
            }
            for property in ["children", "nodes", "items", "options"] {
                if let Some(value) = values.get(property) {
                    collect_tree_node_ids(value, out, seen);
                }
            }
        }
        _ => {}
    }
}

fn preferred_tree_alias_property(
    metadata: &UiTemplateNodeMetadata,
    snake: &'static str,
    camel: &'static str,
) -> &'static str {
    if super::semantics::component_role_is(metadata, "mui-x-tree-view")
        || metadata.attributes.contains_key(camel)
    {
        return camel;
    }
    snake
}

fn find_tree_node_label(value: &toml::Value, target_id: &str) -> Option<String> {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .find_map(|value| find_tree_node_label(value, target_id)),
        toml::Value::String(value) => (value == target_id).then(|| value.clone()),
        toml::Value::Table(values) => {
            if tree_node_identity(values).as_deref() == Some(target_id) {
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

fn tree_node_identity(values: &toml::map::Map<String, toml::Value>) -> Option<String> {
    TREE_NODE_IDENTITY_PROPERTIES
        .iter()
        .filter_map(|property| values.get(*property).and_then(toml::Value::as_str))
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn tree_node_display_text(values: &toml::map::Map<String, toml::Value>) -> Option<String> {
    TREE_NODE_LABEL_PROPERTIES
        .iter()
        .filter_map(|property| values.get(*property).and_then(toml::Value::as_str))
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn collect_borrowed_string_ids<'a>(
    value: &'a toml::Value,
    out: &mut Vec<&'a str>,
    seen: &mut HashSet<&'a str>,
) {
    match value {
        toml::Value::Array(values) => {
            for value in values {
                collect_borrowed_string_ids(value, out, seen);
            }
        }
        toml::Value::String(value) => push_unique_borrowed(out, seen, value),
        toml::Value::Table(values) => {
            for property in OPTION_ID_PROPERTIES {
                if let Some(value) = values.get(property) {
                    collect_borrowed_string_ids(value, out, seen);
                    break;
                }
            }
        }
        _ => {}
    }
}

fn collect_owned_string_ids<'a>(
    value: &'a toml::Value,
    out: &mut Vec<String>,
    seen: &mut HashSet<&'a str>,
) {
    match value {
        toml::Value::Array(values) => {
            for value in values {
                collect_owned_string_ids(value, out, seen);
            }
        }
        toml::Value::String(value) => push_unique_owned(out, seen, value),
        toml::Value::Table(values) => {
            for property in OPTION_ID_PROPERTIES {
                if let Some(value) = values.get(property) {
                    collect_owned_string_ids(value, out, seen);
                    break;
                }
            }
        }
        _ => {}
    }
}

fn push_unique_borrowed<'a>(
    values: &mut Vec<&'a str>,
    seen: &mut HashSet<&'a str>,
    value: &'a str,
) {
    if !value.is_empty() && seen.insert(value) {
        values.push(value);
    }
}

fn push_unique_owned<'a>(values: &mut Vec<String>, seen: &mut HashSet<&'a str>, value: &'a str) {
    if !value.is_empty() && seen.insert(value) {
        values.push(value.to_string());
    }
}

fn value_contains_string(value: &toml::Value, needle: &str) -> bool {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .any(|value| value_contains_string(value, needle)),
        toml::Value::String(value) => value == needle,
        toml::Value::Table(values) => OPTION_ID_PROPERTIES
            .iter()
            .filter_map(|property| values.get(*property))
            .any(|value| value_contains_string(value, needle)),
        _ => false,
    }
}

fn disabled_option_ids(metadata: &UiTemplateNodeMetadata) -> HashSet<&str> {
    let mut ids = HashSet::new();
    if let Some(value) = metadata
        .attributes
        .get("disabled_options")
        .or_else(|| metadata.attributes.get("disabledItems"))
        .or_else(|| metadata.attributes.get("disabled_items"))
    {
        collect_disabled_option_ids(value, &mut ids);
    }
    ids
}

fn collect_disabled_option_ids<'a>(value: &'a toml::Value, out: &mut HashSet<&'a str>) {
    match value {
        toml::Value::Array(values) => {
            for value in values {
                collect_disabled_option_ids(value, out);
            }
        }
        toml::Value::String(value) => {
            out.insert(value);
        }
        toml::Value::Table(values) => {
            for property in OPTION_ID_PROPERTIES {
                if let Some(value) = values.get(property) {
                    collect_disabled_option_ids(value, out);
                }
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

fn role_is_one_of(metadata: &UiTemplateNodeMetadata, roles: &[&str]) -> bool {
    super::semantics::component_role_is_one_of(metadata, roles)
}

fn string_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<String> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> bool {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn int_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<i64> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_integer)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{collect_disabled_option_ids, collect_tree_node_ids};

    #[test]
    fn metadata_tree_ids_borrow_first_occurrence_and_preserve_order() {
        let value = toml::Value::Array(vec![
            toml::Value::String("root".to_string()),
            toml::Value::String("root".to_string()),
            toml::Value::String("child".to_string()),
        ]);
        let first_root = match &value {
            toml::Value::Array(values) => match &values[0] {
                toml::Value::String(value) => value.as_ptr(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let mut ids = Vec::new();
        let mut seen = HashSet::new();

        collect_tree_node_ids(&value, &mut ids, &mut seen);

        assert_eq!(ids, ["root", "child"]);
        assert_eq!(ids[0].as_ptr(), first_root);
    }

    #[test]
    fn metadata_disabled_index_preserves_table_identity_aliases() {
        let value = toml::Value::Array(vec![
            toml::Value::String("root".to_string()),
            toml::Value::Table(
                [(
                    "nodeId".to_string(),
                    toml::Value::String("child".to_string()),
                )]
                .into_iter()
                .collect(),
            ),
        ]);
        let mut disabled = HashSet::new();

        collect_disabled_option_ids(&value, &mut disabled);

        assert_eq!(disabled.len(), 2);
        assert!(disabled.contains("root"));
        assert!(disabled.contains("child"));
    }
}
