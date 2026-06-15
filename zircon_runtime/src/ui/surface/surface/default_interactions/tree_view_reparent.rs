use zircon_runtime_interface::ui::component::UiValue;

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
const TREE_CHILD_PROPERTIES: [&str; 4] = ["children", "nodes", "items", "options"];

pub(super) struct UiTreeReparentedNodes {
    pub values: Vec<UiValue>,
    pub from: usize,
    pub to: usize,
    pub parent_id: String,
}

pub(super) fn reparent_tree_node_values(
    mut values: Vec<UiValue>,
    source_id: &str,
    parent_id: &str,
) -> Option<UiTreeReparentedNodes> {
    if source_id == parent_id || tree_node_contains_descendant(&values, source_id, parent_id) {
        return None;
    }
    let before_ids = flattened_tree_node_ids(&values);
    let from = before_ids.iter().position(|id| id == source_id)?;
    let source = remove_tree_node(&mut values, source_id)?;
    if !insert_tree_node_child(&mut values, parent_id, source) {
        return None;
    }
    let after_ids = flattened_tree_node_ids(&values);
    let to = after_ids.iter().position(|id| id == source_id)?;
    Some(UiTreeReparentedNodes {
        values,
        from,
        to,
        parent_id: parent_id.to_string(),
    })
}

fn remove_tree_node(values: &mut Vec<UiValue>, source_id: &str) -> Option<UiValue> {
    let mut index = 0;
    while index < values.len() {
        if tree_node_id(&values[index]).as_deref() == Some(source_id) {
            return Some(values.remove(index));
        }
        if let UiValue::Map(node) = &mut values[index] {
            for property in TREE_CHILD_PROPERTIES {
                if let Some(UiValue::Array(children)) = node.get_mut(property) {
                    if let Some(removed) = remove_tree_node(children, source_id) {
                        return Some(removed);
                    }
                }
            }
        }
        index += 1;
    }
    None
}

fn insert_tree_node_child(values: &mut [UiValue], parent_id: &str, source: UiValue) -> bool {
    for value in values {
        if let UiValue::Map(node) = value {
            if tree_node_map_id(node).as_deref() == Some(parent_id) {
                let child_property = TREE_CHILD_PROPERTIES
                    .iter()
                    .copied()
                    .find(|property| matches!(node.get(*property), Some(UiValue::Array(_))))
                    .unwrap_or("children");
                let children = node
                    .entry(child_property.to_string())
                    .or_insert_with(|| UiValue::Array(Vec::new()));
                if let UiValue::Array(children) = children {
                    children.push(source);
                    return true;
                }
                return false;
            }
            for property in TREE_CHILD_PROPERTIES {
                if let Some(UiValue::Array(children)) = node.get_mut(property) {
                    if insert_tree_node_child(children, parent_id, source.clone()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn tree_node_contains_descendant(values: &[UiValue], source_id: &str, descendant_id: &str) -> bool {
    for value in values {
        if tree_node_id(value).as_deref() == Some(source_id) {
            return flattened_tree_node_ids_from_value(value)
                .iter()
                .any(|id| id == descendant_id);
        }
        if let UiValue::Map(node) = value {
            for property in TREE_CHILD_PROPERTIES {
                if let Some(UiValue::Array(children)) = node.get(property) {
                    if tree_node_contains_descendant(children, source_id, descendant_id) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn flattened_tree_node_ids(values: &[UiValue]) -> Vec<String> {
    let mut ids = Vec::new();
    for value in values {
        collect_tree_node_ids(value, &mut ids);
    }
    ids
}

fn flattened_tree_node_ids_from_value(value: &UiValue) -> Vec<String> {
    let mut ids = Vec::new();
    collect_tree_node_ids(value, &mut ids);
    ids
}

fn collect_tree_node_ids(value: &UiValue, out: &mut Vec<String>) {
    if let Some(id) = tree_node_id(value) {
        out.push(id);
    }
    if let UiValue::Map(node) = value {
        for property in TREE_CHILD_PROPERTIES {
            if let Some(UiValue::Array(children)) = node.get(property) {
                for child in children {
                    collect_tree_node_ids(child, out);
                }
            }
        }
    }
}

fn tree_node_id(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value.clone()),
        UiValue::Map(node) => tree_node_map_id(node),
        _ => None,
    }
}

fn tree_node_map_id(values: &std::collections::BTreeMap<String, UiValue>) -> Option<String> {
    TREE_NODE_IDENTITY_PROPERTIES
        .iter()
        .filter_map(|property| values.get(*property).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}
