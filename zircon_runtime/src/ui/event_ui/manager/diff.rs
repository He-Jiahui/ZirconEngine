use zircon_runtime_interface::ui::event_ui::{UiReflectionDiff, UiReflectionSnapshot};

#[cfg(test)]
#[path = "diff/tests/capacity_tests.rs"]
mod capacity_tests;

// 同一 tree 的替换快照按 node_id 比较；结果只通知哪些节点重查，不包含属性补丁或真实表面变更。
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
