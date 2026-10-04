use super::{
    projection_node_change_reasons, projection_node_map, projection_update_domains,
    UiEcsDirtyDomains, UiEcsNodeProjection, UiEcsProjectionChangeKind, UiEcsProjectionChangeReason,
    UiEcsProjectionNodeChange,
};

pub(super) fn projection_changes(
    previous: &[UiEcsNodeProjection],
    current: &[UiEcsNodeProjection],
) -> Vec<UiEcsProjectionNodeChange> {
    // 唯一递增 ID 可用双指针线性合并；乱序或重复 ID 回退 BTreeMap，以保留重复键覆盖和排序语义。
    if nodes_are_strictly_ordered(previous) && nodes_are_strictly_ordered(current) {
        ordered_projection_changes(previous, current)
    } else {
        mapped_projection_changes(previous, current)
    }
}

fn nodes_are_strictly_ordered(nodes: &[UiEcsNodeProjection]) -> bool {
    nodes
        .windows(2)
        .all(|pair| pair[0].node_id < pair[1].node_id)
}

fn ordered_projection_changes(
    previous: &[UiEcsNodeProjection],
    current: &[UiEcsNodeProjection],
) -> Vec<UiEcsProjectionNodeChange> {
    let mut changes = Vec::new();
    let mut current_index = 0;
    for previous_node in previous {
        while current
            .get(current_index)
            .is_some_and(|current_node| current_node.node_id < previous_node.node_id)
        {
            current_index += 1;
        }
        let current_node = current
            .get(current_index)
            .filter(|current_node| current_node.node_id == previous_node.node_id);
        if let Some(change) = updated_or_removed_change(previous_node, current_node) {
            changes.push(change);
        }
    }

    let mut previous_index = 0;
    for current_node in current {
        while previous
            .get(previous_index)
            .is_some_and(|previous_node| previous_node.node_id < current_node.node_id)
        {
            previous_index += 1;
        }
        let already_existed = previous
            .get(previous_index)
            .is_some_and(|previous_node| previous_node.node_id == current_node.node_id);
        if !already_existed {
            changes.push(added_change(current_node));
        }
    }
    changes
}

fn mapped_projection_changes(
    previous: &[UiEcsNodeProjection],
    current: &[UiEcsNodeProjection],
) -> Vec<UiEcsProjectionNodeChange> {
    let previous_nodes = projection_node_map(previous);
    let current_nodes = projection_node_map(current);
    let mut changes = Vec::new();

    for (node_id, previous_node) in &previous_nodes {
        if let Some(change) =
            updated_or_removed_change(previous_node, current_nodes.get(node_id).copied())
        {
            changes.push(change);
        }
    }
    for (node_id, current_node) in &current_nodes {
        if !previous_nodes.contains_key(node_id) {
            changes.push(added_change(current_node));
        }
    }
    changes
}

fn updated_or_removed_change(
    previous: &UiEcsNodeProjection,
    current: Option<&UiEcsNodeProjection>,
) -> Option<UiEcsProjectionNodeChange> {
    let Some(current) = current else {
        return Some(UiEcsProjectionNodeChange {
            node_id: previous.node_id,
            node_path: previous.node_path.clone(),
            kind: UiEcsProjectionChangeKind::Removed,
            domains: UiEcsDirtyDomains::structural_change().union(previous.dirty),
            reasons: vec![UiEcsProjectionChangeReason::Removed],
        });
    };

    let reasons = projection_node_change_reasons(previous, current);
    (!reasons.is_empty()).then(|| UiEcsProjectionNodeChange {
        node_id: previous.node_id,
        node_path: current.node_path.clone(),
        kind: UiEcsProjectionChangeKind::Updated,
        domains: projection_update_domains(previous, current, &reasons),
        reasons,
    })
}

fn added_change(current: &UiEcsNodeProjection) -> UiEcsProjectionNodeChange {
    UiEcsProjectionNodeChange {
        node_id: current.node_id,
        node_path: current.node_path.clone(),
        kind: UiEcsProjectionChangeKind::Added,
        domains: UiEcsDirtyDomains::structural_change().union(current.dirty),
        reasons: vec![UiEcsProjectionChangeReason::Added],
    }
}

#[cfg(test)]
#[path = "tests/diff.rs"]
mod tests;
