use zircon_runtime_interface::ui::event_ui::{UiReflectionDiff, UiReflectionSnapshot};

#[cfg(test)]
mod capacity_tests;

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
