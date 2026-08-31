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
    let mut route_nodes = arranged_tree
        .nodes
        .iter()
        .map(|node| UiHitRouteNode::invalid(node.node_id))
        .collect::<Vec<_>>();
    let mut states = vec![RouteBuildState::Unresolved; route_nodes.len()];

    for start_index in 0..route_nodes.len() {
        if states[start_index] != RouteBuildState::Unresolved {
            continue;
        }
        let mut chain = Vec::new();
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
            chain.push(index);
            current_index = match node.parent {
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
        }

        if failed {
            invalidate_chain(&mut route_nodes, &mut states, chain);
            continue;
        }
        while let Some(index) = chain.pop() {
            let Some(node) = arranged_tree.nodes.get(index) else {
                states[index] = RouteBuildState::Invalid;
                continue;
            };
            let next = compose_route_node(node, node_indices, &route_nodes, None);
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
        let next = compose_route_node(node, node_indices, route_nodes, Some(&updates));
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
    for depth in 0..=grid.route_nodes.len() {
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
    for depth in 0..=grid.route_nodes.len() {
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
    chain: Vec<usize>,
) {
    for index in chain {
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
) -> UiHitRouteNode {
    let (parent_index, inherited_input_policy, inherited_pointer_visibility) = match node.parent {
        Some(parent_id) => {
            let Some(parent_index) = node_indices.get(&parent_id).copied() else {
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
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        layout::UiFrame,
        tree::{UiPointerEvents, UiVisibility},
    };

    #[test]
    fn deep_chain_builds_without_recursion() {
        const NODE_COUNT: usize = 4_096;
        let mut nodes = Vec::with_capacity(NODE_COUNT);
        for index in 0..NODE_COUNT {
            let node_id = UiNodeId::new((index + 1) as u64);
            let parent = (index > 0).then(|| UiNodeId::new(index as u64));
            let mut node = pointer_node(node_id, parent);
            if index + 1 < NODE_COUNT {
                node.children.push(UiNodeId::new((index + 2) as u64));
            }
            nodes.push(node);
        }
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.route.deep"),
            roots: vec![UiNodeId::new(1)].into(),
            draw_order: nodes
                .iter()
                .map(|node| node.node_id)
                .collect::<Vec<_>>()
                .into(),
            nodes: nodes.into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = arranged_tree
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.node_id, index))
            .collect();
        let route_nodes = build_route_nodes(&arranged_tree, &node_indices);
        let leaf_id = UiNodeId::new(NODE_COUNT as u64);
        let entry = UiHitTestEntry {
            node_id: leaf_id,
            frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            clip_frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            z_index: 0,
            paint_order: 0,
            control_id: None,
            route_node_index: (NODE_COUNT - 1) as u32,
        };
        let grid = UiHitTestGrid {
            route_nodes,
            entries: vec![entry.clone()].into(),
            ..UiHitTestGrid::default()
        };

        let route = bubble_route_for_entry(&grid, &entry).expect("deep route must resolve");
        assert_eq!(route.len(), NODE_COUNT);
        assert_eq!(route.first(), Some(&leaf_id));
        assert_eq!(route.last(), Some(&UiNodeId::new(1)));
    }

    #[test]
    fn missing_parent_and_cycle_fail_closed() {
        let missing_id = UiNodeId::new(1);
        let missing_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.route.missing"),
            roots: Vec::new().into(),
            nodes: vec![pointer_node(missing_id, Some(UiNodeId::new(99)))].into(),
            draw_order: vec![missing_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let missing_routes = build_route_nodes(&missing_tree, &BTreeMap::from([(missing_id, 0)]));
        assert!(!missing_routes[0].route_valid);

        let first_id = UiNodeId::new(10);
        let second_id = UiNodeId::new(11);
        let mut first = pointer_node(first_id, Some(second_id));
        first.children.push(second_id);
        let mut second = pointer_node(second_id, Some(first_id));
        second.children.push(first_id);
        let cycle_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.route.cycle"),
            roots: Vec::new().into(),
            nodes: vec![first, second].into(),
            draw_order: vec![first_id, second_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let cycle_routes = build_route_nodes(
            &cycle_tree,
            &BTreeMap::from([(first_id, 0), (second_id, 1)]),
        );
        assert!(cycle_routes.iter().all(|route| !route.route_valid));
    }

    #[test]
    fn input_patch_updates_descendant_route_semantics() {
        let parent_id = UiNodeId::new(20);
        let child_id = UiNodeId::new(21);
        let mut parent = pointer_node(parent_id, None);
        parent.children.push(child_id);
        parent.input_policy = UiInputPolicy::Receive;
        let child = pointer_node(child_id, Some(parent_id));
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.route.input-patch"),
            roots: vec![parent_id].into(),
            nodes: vec![parent, child].into(),
            draw_order: vec![parent_id, child_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(parent_id, 0), (child_id, 1)]);
        let mut route_nodes = build_route_nodes(&arranged_tree, &node_indices);
        assert_eq!(
            route_nodes[1].effective_input_policy,
            UiInputPolicy::Receive
        );

        arranged_tree.nodes[0].input_policy = UiInputPolicy::Ignore;
        assert_eq!(
            patch_route_nodes(
                &mut route_nodes,
                &arranged_tree,
                &BTreeSet::from([parent_id, child_id]),
                &node_indices,
            ),
            Ok(true)
        );
        assert_eq!(route_nodes[1].effective_input_policy, UiInputPolicy::Ignore);
    }

    #[test]
    fn input_patch_without_route_semantic_change_keeps_shared_allocation() {
        let node_id = UiNodeId::new(30);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.route.noop-input-patch"),
            roots: vec![node_id].into(),
            nodes: vec![pointer_node(node_id, None)].into(),
            draw_order: vec![node_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(node_id, 0)]);
        let mut route_nodes = build_route_nodes(&arranged_tree, &node_indices);
        let shared = route_nodes.clone();

        assert_eq!(
            patch_route_nodes(
                &mut route_nodes,
                &arranged_tree,
                &BTreeSet::from([node_id]),
                &node_indices,
            ),
            Ok(false)
        );
        assert!(Arc::ptr_eq(&route_nodes, &shared));
    }

    fn pointer_node(node_id: UiNodeId, parent: Option<UiNodeId>) -> UiArrangedNode {
        UiArrangedNode {
            node_id,
            node_path: UiNodePath::new(format!("root/{}", node_id.0)),
            parent,
            children: Vec::new(),
            frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            clip_frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            z_index: 0,
            paint_order: node_id.0,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Inherit,
            pointer_events: UiPointerEvents::Auto,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }
    }
}
