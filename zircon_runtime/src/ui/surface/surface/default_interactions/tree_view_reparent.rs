use zircon_runtime_interface::ui::component::UiValue;

// 层级数据的身份/子数组别名须与树元数据解析约定配套；调用方负责提供唯一且稳定的项标识。
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

/// 新层级及前序位置回执；from/to 是遍历位置，二者相等仍可能发生真实父级变更。
pub(super) struct UiTreeReparentedNodes {
    pub values: Vec<UiValue>,
    pub from: usize,
    pub to: usize,
    pub parent_id: String,
}

/// 对调用方交付的独占模型副本重挂节点，先拒绝自挂与祖先循环；失败不会发布已局部修改的副本。
/// 只操作模型数组/映射，不修改 UiTree 的保留节点关系，最终提交及选择/展开同步属于 Surface 调用者。
pub(super) fn reparent_tree_node_values(
    mut values: Vec<UiValue>,
    source_id: &str,
    parent_id: &str,
) -> Option<UiTreeReparentedNodes> {
    if source_id == parent_id || tree_node_contains_descendant(&values, source_id, parent_id) {
        return None;
    }
    let from = flattened_tree_node_position(&values, source_id)?;
    let mut source = Some(remove_tree_node(&mut values, source_id)?);
    if !insert_tree_node_child(&mut values, parent_id, &mut source) {
        return None;
    }
    let to = flattened_tree_node_position(&values, source_id)?;
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
        if tree_node_id(&values[index]) == Some(source_id) {
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

fn insert_tree_node_child(
    values: &mut [UiValue],
    parent_id: &str,
    source: &mut Option<UiValue>,
) -> bool {
    for value in values {
        if let UiValue::Map(node) = value {
            if tree_node_map_id(node) == Some(parent_id) {
                let child_property = TREE_CHILD_PROPERTIES
                    .iter()
                    .copied()
                    .find(|property| matches!(node.get(*property), Some(UiValue::Array(_))))
                    .unwrap_or("children");
                let children = node
                    .entry(child_property.to_string())
                    .or_insert_with(|| UiValue::Array(Vec::new()));
                if let UiValue::Array(children) = children {
                    let Some(source) = source.take() else {
                        return false;
                    };
                    children.push(source);
                    return true;
                }
                return false;
            }
            for property in TREE_CHILD_PROPERTIES {
                if let Some(UiValue::Array(children)) = node.get_mut(property) {
                    if insert_tree_node_child(children, parent_id, source) {
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
        if tree_node_id(value) == Some(source_id) {
            return tree_node_value_contains_id(value, descendant_id);
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

fn tree_node_value_contains_id(value: &UiValue, target_id: &str) -> bool {
    if tree_node_id(value) == Some(target_id) {
        return true;
    }
    if let UiValue::Map(node) = value {
        for property in TREE_CHILD_PROPERTIES {
            if let Some(UiValue::Array(children)) = node.get(property) {
                if children
                    .iter()
                    .any(|child| tree_node_value_contains_id(child, target_id))
                {
                    return true;
                }
            }
        }
    }
    false
}

fn flattened_tree_node_position(values: &[UiValue], target_id: &str) -> Option<usize> {
    let mut visited = 0;
    for value in values {
        if let Some(position) = tree_node_value_position(value, target_id, &mut visited) {
            return Some(position);
        }
    }
    None
}

fn tree_node_value_position(
    value: &UiValue,
    target_id: &str,
    visited: &mut usize,
) -> Option<usize> {
    if let Some(id) = tree_node_id(value) {
        if id == target_id {
            return Some(*visited);
        }
        *visited = visited.saturating_add(1);
    }
    if let UiValue::Map(node) = value {
        for property in TREE_CHILD_PROPERTIES {
            if let Some(UiValue::Array(children)) = node.get(property) {
                for child in children {
                    if let Some(position) = tree_node_value_position(child, target_id, visited) {
                        return Some(position);
                    }
                }
            }
        }
    }
    None
}

fn tree_node_id(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value),
        UiValue::Map(node) => tree_node_map_id(node),
        _ => None,
    }
}

fn tree_node_map_id(values: &std::collections::BTreeMap<String, UiValue>) -> Option<&str> {
    TREE_NODE_IDENTITY_PROPERTIES
        .iter()
        .filter_map(|property| values.get(*property).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tree_view_reparent/tests/borrowed_traversal_tests.rs"]
mod borrowed_traversal_tests;
