use super::super::{UiReflectionDiff, UiReflectionSnapshot};

pub(crate) fn compute_diff(
    previous: &UiReflectionSnapshot,
    current: &UiReflectionSnapshot,
) -> UiReflectionDiff {
    let mut changed_nodes = Vec::new();
    let mut removed_nodes = Vec::new();

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
