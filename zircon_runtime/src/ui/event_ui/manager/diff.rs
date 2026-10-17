use zircon_runtime_interface::ui::event_ui::{UiReflectionDiff, UiReflectionSnapshot};

#[cfg(test)]
#[path = "diff/tests/capacity_tests.rs"]
mod capacity_tests;

// 同一 tree 的替换快照按 node_id 比较；结果只通知哪些节点重查，不包含属性补丁或真实表面变更。
/// 按节点描述比较两个快照，生成需要重查和移除的节点编号。
///
/// # 参数与返回
/// 两个快照仅在调用期间借用；新增或描述变化的节点进入变化列表，旧有但当前缺失的节点进入移除列表。
/// 两列表分别沿节点映射的编号顺序生成，结果拥有编号及 `current` 的树身份。
///
/// # 约束
/// 同树前提由 `UiEventManager::replace_tree` 的树键保证；本函数不校验两个树身份，也不比较根列表。
pub(crate) fn compute_diff(
    previous: &UiReflectionSnapshot,
    current: &UiReflectionSnapshot,
) -> UiReflectionDiff {
    let (changed_capacity, removed_capacity) = reflection_diff_capacities(previous, current);
    let mut changed_nodes = Vec::with_capacity(changed_capacity);
    let mut removed_nodes = Vec::with_capacity(removed_capacity);

    for (node_id, node) in &current.nodes {
        if previous.nodes.get(node_id) != Some(node) {
            changed_nodes.push(*node_id);
        }
    }
    for node_id in previous.nodes.keys() {
        if !current.nodes.contains_key(node_id) {
            removed_nodes.push(*node_id);
        }
    }

    UiReflectionDiff {
        tree_id: current.tree_id.clone(),
        changed_nodes,
        removed_nodes,
    }
}

// 预扫描与输出循环使用相同谓词，为两组编号分别计算容量需求；查表和比较仍会在两轮遍历中执行。
fn reflection_diff_capacities(
    previous: &UiReflectionSnapshot,
    current: &UiReflectionSnapshot,
) -> (usize, usize) {
    let changed = current
        .nodes
        .iter()
        .filter(|(node_id, node)| previous.nodes.get(*node_id) != Some(*node))
        .count();
    let removed = previous
        .nodes
        .keys()
        .filter(|node_id| !current.nodes.contains_key(*node_id))
        .count();
    (changed, removed)
}
