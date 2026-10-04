use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiArrangedNode, UiArrangedTree, UiHitRouteNode, UiHitTestEntry, UiHitTestGrid},
    tree::UiInputPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteBuildState {
    Unresolved,
    Visiting,
    Resolved,
    Invalid,
}

pub(super) fn build_route_nodes(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
) -> Arc<Vec<UiHitRouteNode>> {
    let mut route_nodes = Vec::with_capacity(arranged_tree.nodes.len());
    route_nodes.extend(
        arranged_tree
            .nodes
            .iter()
            .map(|node| UiHitRouteNode::invalid(node.node_id)),
    );
    let mut states = vec![RouteBuildState::Unresolved; route_nodes.len()];

    // Keep one traversal scratch buffer for the whole publication.  A tree with
    // many roots (or disconnected components) used to allocate a fresh `Vec`
    // for every start node even though each chain is consumed before the next
    // one begins.  Reusing the buffer preserves the iterative/depth-bounded
    // walk while making steady-state route publication allocation-free after
    // the deepest chain has established its capacity.
    let mut chain: Vec<(usize, Option<usize>)> = Vec::new();
    for start_index in 0..route_nodes.len() {
        if states[start_index] != RouteBuildState::Unresolved {
            continue;
        }
        chain.clear();
        let mut current_index = Some(start_index);
        let mut failed = false;
        while let Some(index) = current_index {
            match states.get(index).copied() {
                Some(RouteBuildState::Resolved) => break,
                Some(RouteBuildState::Invalid | RouteBuildState::Visiting) | None => {
                    failed = true;
                    break;
                }
                Some(RouteBuildState::Unresolved) => {}
            }
            let Some(node) = arranged_tree.nodes.get(index) else {
                failed = true;
                break;
            };
            if node_indices.get(&node.node_id).copied() != Some(index) {
                failed = true;
                break;
            }
            states[index] = RouteBuildState::Visiting;
            let parent_index = match node.parent {
                Some(parent_id) => match node_indices.get(&parent_id).copied() {
                    Some(parent_index)
                        if arranged_tree
                            .nodes
                            .get(parent_index)
                            .is_some_and(|parent| parent.node_id == parent_id) =>
                    {
                        Some(parent_index)
                    }
                    _ => {
                        failed = true;
                        None
                    }
                },
                None => None,
            };
            current_index = parent_index;
            chain.push((index, parent_index));
        }

        if failed {
            invalidate_chain(&mut route_nodes, &mut states, &chain);
            continue;
        }
        while let Some((index, parent_index)) = chain.pop() {
            let Some(node) = arranged_tree.nodes.get(index) else {
                states[index] = RouteBuildState::Invalid;
                continue;
            };
            let next = compose_route_node(node, node_indices, &route_nodes, None, parent_index);
            states[index] = if next.route_valid {
                RouteBuildState::Resolved
            } else {
                RouteBuildState::Invalid
            };
            route_nodes[index] = next;
        }
    }

    Arc::new(route_nodes)
}

pub(super) fn patch_route_nodes(
    route_nodes: &mut Arc<Vec<UiHitRouteNode>>,
    arranged_tree: &UiArrangedTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
    node_indices: &BTreeMap<UiNodeId, usize>,
) -> Result<bool, ()> {
    if changed_node_ids.is_empty() {
        return Ok(false);
    }
    if route_nodes.len() != arranged_tree.nodes.len() {
        return Err(());
    }
    let mut affected_indices = BTreeSet::new();
    for node_id in changed_node_ids {
        let index = node_indices.get(node_id).copied().ok_or(())?;
        let node = arranged_tree.nodes.get(index).ok_or(())?;
        if node.node_id != *node_id
            || route_nodes.get(index).map(|route| route.node_id) != Some(*node_id)
        {
            return Err(());
        }
        affected_indices.insert(index);
    }

    let mut ready = VecDeque::new();
    for index in &affected_indices {
        let node = arranged_tree.nodes.get(*index).ok_or(())?;
        let parent_is_affected = node
            .parent
            .and_then(|parent_id| node_indices.get(&parent_id).copied())
            .is_some_and(|parent_index| affected_indices.contains(&parent_index));
        if !parent_is_affected {
            ready.push_back(*index);
        }
    }

    let mut processed = 0usize;
    let mut changed = false;
    let mut updates = BTreeMap::new();
    while let Some(index) = ready.pop_front() {
        let node = arranged_tree.nodes.get(index).ok_or(())?;
        let next = compose_route_node(node, node_indices, route_nodes, Some(&updates), None);
        changed |= route_nodes.get(index) != Some(&next);
        updates.insert(index, next);
        processed = processed.saturating_add(1);
        for child_id in &node.children {
            let child_index = node_indices.get(child_id).copied().ok_or(())?;
            let child = arranged_tree.nodes.get(child_index).ok_or(())?;
            if child.node_id != *child_id || child.parent != Some(node.node_id) {
                return Err(());
            }
            if affected_indices.contains(&child_index) {
                ready.push_back(child_index);
            }
        }
    }

    if processed != affected_indices.len() {
        return Err(());
    }
    if changed {
        let route_nodes = Arc::make_mut(route_nodes);
        for (index, route) in updates {
            route_nodes[index] = route;
        }
    }
    Ok(changed)
}

pub(super) fn route_node_index_for_node(
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Option<u32> {
    let index = u32::try_from(*node_indices.get(&node_id)?).ok()?;
    (index != UiHitRouteNode::NO_PARENT_INDEX).then_some(index)
}

pub(super) fn route_node_for_entry<'a>(
    grid: &'a UiHitTestGrid,
    entry: &UiHitTestEntry,
) -> Option<&'a UiHitRouteNode> {
    let route = grid.route_nodes.get(entry.route_node_index as usize)?;
    (route.route_valid && route.node_id == entry.node_id).then_some(route)
}

pub(super) fn bubble_route_for_entry(
    grid: &UiHitTestGrid,
    entry: &UiHitTestEntry,
) -> Option<Vec<UiNodeId>> {
    let mut route = Vec::new();
    let mut route_index = entry.route_node_index;
    // Bound the walk by the number of nodes; a tree can be at most that deep.
    // Use `<` (not `<=`) so a tree whose depth equals node count still returns
    // rather than falling through to None.
    for depth in 0..grid.route_nodes.len() {
        let node = grid.route_nodes.get(route_index as usize)?;
        if !node.route_valid || (depth == 0 && node.node_id != entry.node_id) {
            return None;
        }
        route.push(node.node_id);
        let Some(parent_index) = node.parent_index() else {
            return Some(route);
        };
        route_index = u32::try_from(parent_index).ok()?;
    }
    None
}

pub(crate) fn find_bubble_route_value<T: Copy>(
    grid: &UiHitTestGrid,
    entry: &UiHitTestEntry,
    values: &BTreeMap<UiNodeId, T>,
) -> Option<T> {
    let mut route_index = entry.route_node_index;
    for depth in 0..grid.route_nodes.len() {
        let node = grid.route_nodes.get(route_index as usize)?;
        if !node.route_valid || (depth == 0 && node.node_id != entry.node_id) {
            return None;
        }
        if let Some(value) = values.get(&node.node_id) {
            return Some(*value);
        }
        let Some(parent_index) = node.parent_index() else {
            return None;
        };
        route_index = u32::try_from(parent_index).ok()?;
    }
    None
}

fn invalidate_chain(
    route_nodes: &mut [UiHitRouteNode],
    states: &mut [RouteBuildState],
    chain: &[(usize, Option<usize>)],
) {
    for &(index, _) in chain {
        if let Some(route) = route_nodes.get_mut(index) {
            *route = UiHitRouteNode::invalid(route.node_id);
        }
        if let Some(state) = states.get_mut(index) {
            *state = RouteBuildState::Invalid;
        }
    }
}

fn compose_route_node(
    node: &UiArrangedNode,
    node_indices: &BTreeMap<UiNodeId, usize>,
    route_nodes: &[UiHitRouteNode],
    updates: Option<&BTreeMap<usize, UiHitRouteNode>>,
    parent_index_hint: Option<usize>,
) -> UiHitRouteNode {
    let (parent_index, inherited_input_policy, inherited_pointer_visibility) = match node.parent {
        Some(parent_id) => {
            let Some(parent_index) =
                parent_index_hint.or_else(|| node_indices.get(&parent_id).copied())
            else {
                return UiHitRouteNode::invalid(node.node_id);
            };
            let Some(parent) = updates
                .and_then(|updates| updates.get(&parent_index))
                .or_else(|| route_nodes.get(parent_index))
            else {
                return UiHitRouteNode::invalid(node.node_id);
            };
            let Ok(parent_index) = u32::try_from(parent_index) else {
                return UiHitRouteNode::invalid(node.node_id);
            };
            if parent_index == UiHitRouteNode::NO_PARENT_INDEX {
                return UiHitRouteNode::invalid(node.node_id);
            }
            if !parent.route_valid || parent.node_id != parent_id {
                return UiHitRouteNode::invalid(node.node_id);
            }
            (
                parent_index,
                parent.effective_input_policy,
                parent.descendant_pointer_path_visible,
            )
        }
        None => (
            UiHitRouteNode::NO_PARENT_INDEX,
            UiInputPolicy::Receive,
            true,
        ),
    };
    let effective_input_policy = match node.input_policy {
        UiInputPolicy::Inherit => inherited_input_policy,
        explicit => explicit,
    };
    UiHitRouteNode {
        node_id: node.node_id,
        parent_index,
        effective_input_policy,
        pointer_path_visible: inherited_pointer_visibility && node.allows_self_pointer_hit_test(),
        descendant_pointer_path_visible: inherited_pointer_visibility
            && node.allows_child_pointer_hit_test(),
        route_valid: true,
    }
}

#[cfg(test)]
#[path = "tests/route_index.rs"]
mod tests;
