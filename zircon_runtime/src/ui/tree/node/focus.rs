use std::cmp::Ordering;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationGroupId, UiTabIndex,
    },
    surface::UiNavigationEventKind,
    tree::{UiTree, UiTreeError},
};

pub trait UiRuntimeTreeFocusExt {
    fn first_focusable_in_route(&self, route: &[UiNodeId])
        -> Result<Option<UiNodeId>, UiTreeError>;
    fn focusable_nodes_in_navigation_order(&self) -> Result<Vec<UiNodeId>, UiTreeError>;
    fn next_focusable_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError>;

    fn next_navigation_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError>;
}

impl UiRuntimeTreeFocusExt for UiTree {
    fn first_focusable_in_route(
        &self,
        route: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        for node_id in route {
            let node = self
                .nodes
                .get(node_id)
                .ok_or(UiTreeError::MissingNode(*node_id))?;
            if node.is_focus_candidate() {
                return Ok(Some(*node_id));
            }
        }
        Ok(None)
    }

    fn focusable_nodes_in_navigation_order(&self) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut focusable = Vec::new();
        for root_id in &self.roots {
            collect_focusable_nodes(self, *root_id, &mut focusable)?;
        }
        Ok(focusable)
    }

    fn next_focusable_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let focusable = self.focusable_nodes_in_navigation_order()?;
        if focusable.is_empty() {
            return Ok(None);
        }

        match kind {
            UiNavigationEventKind::Next => {
                if let Some(current) = current {
                    if let Some(index) = focusable.iter().position(|node_id| *node_id == current) {
                        return Ok(Some(focusable[(index + 1) % focusable.len()]));
                    }
                }
                Ok(focusable.first().copied())
            }
            UiNavigationEventKind::Previous => {
                if let Some(current) = current {
                    if let Some(index) = focusable.iter().position(|node_id| *node_id == current) {
                        let previous_index = if index == 0 {
                            focusable.len() - 1
                        } else {
                            index - 1
                        };
                        return Ok(Some(focusable[previous_index]));
                    }
                }
                Ok(focusable.last().copied())
            }
            UiNavigationEventKind::Up
            | UiNavigationEventKind::Down
            | UiNavigationEventKind::Left
            | UiNavigationEventKind::Right => {
                let Some(current) = current else {
                    return Ok(match kind {
                        UiNavigationEventKind::Up | UiNavigationEventKind::Left => {
                            focusable.last().copied()
                        }
                        UiNavigationEventKind::Down | UiNavigationEventKind::Right => {
                            focusable.first().copied()
                        }
                        UiNavigationEventKind::Activate
                        | UiNavigationEventKind::Cancel
                        | UiNavigationEventKind::Home
                        | UiNavigationEventKind::End
                        | UiNavigationEventKind::Next
                        | UiNavigationEventKind::Previous => None,
                    });
                };
                nearest_focusable_in_direction(self, current, kind, &focusable)
            }
            UiNavigationEventKind::Activate
            | UiNavigationEventKind::Cancel
            | UiNavigationEventKind::Home
            | UiNavigationEventKind::End => Ok(None),
        }
    }

    fn next_navigation_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        match kind {
            UiNavigationEventKind::Next | UiNavigationEventKind::Previous => {
                next_tab_target(self, current, kind)
            }
            UiNavigationEventKind::Up
            | UiNavigationEventKind::Down
            | UiNavigationEventKind::Left
            | UiNavigationEventKind::Right => next_directional_target(self, current, kind),
            UiNavigationEventKind::Activate
            | UiNavigationEventKind::Cancel
            | UiNavigationEventKind::Home
            | UiNavigationEventKind::End => Ok(None),
        }
    }
}

fn next_tab_target(
    tree: &UiTree,
    current: Option<UiNodeId>,
    kind: UiNavigationEventKind,
) -> Result<Option<UiNodeId>, UiTreeError> {
    let candidates = tab_candidates(tree, current)?;
    if candidates.is_empty() {
        return Ok(None);
    }
    let index = current.and_then(|current| {
        candidates
            .iter()
            .position(|candidate| candidate.node_id == current)
    });
    let next_index = match (kind, index) {
        (UiNavigationEventKind::Next, Some(index)) => (index + 1) % candidates.len(),
        (UiNavigationEventKind::Previous, Some(0)) => candidates.len() - 1,
        (UiNavigationEventKind::Previous, Some(index)) => index - 1,
        (UiNavigationEventKind::Next, None) => 0,
        (UiNavigationEventKind::Previous, None) => candidates.len() - 1,
        _ => return Ok(None),
    };
    Ok(Some(candidates[next_index].node_id))
}

fn next_directional_target(
    tree: &UiTree,
    current: Option<UiNodeId>,
    kind: UiNavigationEventKind,
) -> Result<Option<UiNodeId>, UiTreeError> {
    let Some(current) = current else {
        return spatial_direction_target(tree, current, kind);
    };
    let modal_group = modal_group_for(tree, current);
    if let Some(target) = manual_direction_target(tree, current, kind)? {
        return Ok(filter_modal_target(tree, target, modal_group.as_ref()));
    }
    spatial_direction_target(tree, Some(current), kind)
}

fn manual_direction_target(
    tree: &UiTree,
    current: UiNodeId,
    kind: UiNavigationEventKind,
) -> Result<Option<Option<UiNodeId>>, UiTreeError> {
    let node = tree
        .nodes
        .get(&current)
        .ok_or(UiTreeError::MissingNode(current))?;
    let Some(directional) = node.navigation.directional.as_ref() else {
        return Ok(None);
    };
    match target_for_direction(directional, kind) {
        UiDirectionalNavigationTarget::Auto => Ok(None),
        UiDirectionalNavigationTarget::Blocked => Ok(Some(None)),
        UiDirectionalNavigationTarget::Node(node_id) => Ok(Some(
            tree.nodes
                .get(node_id)
                .filter(|node| node.is_focus_candidate())
                .map(|_| *node_id),
        )),
        UiDirectionalNavigationTarget::Group(group_id) => {
            Ok(Some(first_candidate_in_group(tree, group_id)))
        }
    }
}

fn spatial_direction_target(
    tree: &UiTree,
    current: Option<UiNodeId>,
    kind: UiNavigationEventKind,
) -> Result<Option<UiNodeId>, UiTreeError> {
    let candidates = spatial_candidates(tree, current)?;
    if candidates.is_empty() {
        return Ok(None);
    }
    let Some(current) = current else {
        return Ok(match kind {
            UiNavigationEventKind::Up | UiNavigationEventKind::Left => {
                candidates.last().map(|candidate| candidate.node_id)
            }
            UiNavigationEventKind::Down | UiNavigationEventKind::Right => {
                candidates.first().map(|candidate| candidate.node_id)
            }
            _ => None,
        });
    };
    nearest_navigation_candidate_in_direction(tree, current, kind, &candidates)
}

fn tab_candidates(
    tree: &UiTree,
    current: Option<UiNodeId>,
) -> Result<Vec<NavigationCandidate>, UiTreeError> {
    let modal_group = current.and_then(|current| modal_group_for(tree, current));
    let mut candidates = navigation_candidates(tree)?;
    candidates.retain(|candidate| {
        if !candidate.tabbable {
            return false;
        }
        match modal_group.as_ref() {
            Some(group_id) => candidate.group_id.as_ref() == Some(group_id),
            None => !candidate.modal,
        }
    });
    candidates.sort_by(compare_tab_candidates);
    Ok(candidates)
}

fn spatial_candidates(
    tree: &UiTree,
    current: Option<UiNodeId>,
) -> Result<Vec<NavigationCandidate>, UiTreeError> {
    let modal_group = current.and_then(|current| modal_group_for(tree, current));
    let mut candidates = navigation_candidates(tree)?;
    candidates.retain(|candidate| {
        modal_group.as_ref().map_or(true, |group_id| {
            candidate.group_id.as_ref() == Some(group_id)
        })
    });
    candidates.sort_by(compare_tree_candidates);
    Ok(candidates)
}

fn navigation_candidates(tree: &UiTree) -> Result<Vec<NavigationCandidate>, UiTreeError> {
    let mut candidates = Vec::new();
    for root_id in &tree.roots {
        collect_navigation_candidates(tree, *root_id, &mut candidates)?;
    }
    Ok(candidates)
}

fn collect_navigation_candidates(
    tree: &UiTree,
    node_id: UiNodeId,
    candidates: &mut Vec<NavigationCandidate>,
) -> Result<(), UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if node.is_focus_candidate() {
        let tab_index = node.navigation.tab_index.unwrap_or_else(|| UiTabIndex {
            order: node.paint_order.min(i32::MAX as u64) as i32,
            tabbable: node.is_focus_candidate(),
        });
        let group = node.navigation.group.as_ref();
        candidates.push(NavigationCandidate {
            node_id,
            tab_order: tab_index.order,
            tabbable: tab_index.tabbable,
            group_order: group.map_or(0, |group| group.order),
            group_id: group.map(|group| group.group_id.clone()),
            modal: group.is_some_and(|group| group.modal),
            paint_order: node.paint_order,
        });
    }
    for child_id in &node.children {
        collect_navigation_candidates(tree, *child_id, candidates)?;
    }
    Ok(())
}

fn modal_group_for(tree: &UiTree, node_id: UiNodeId) -> Option<UiNavigationGroupId> {
    let mut current = Some(node_id);
    while let Some(node_id) = current {
        let node = tree.nodes.get(&node_id)?;
        if let Some(group) = node.navigation.group.as_ref().filter(|group| group.modal) {
            return Some(group.group_id.clone());
        }
        current = node.parent;
    }
    None
}

fn first_candidate_in_group(tree: &UiTree, group_id: &UiNavigationGroupId) -> Option<UiNodeId> {
    let mut candidates = navigation_candidates(tree).ok()?;
    candidates.retain(|candidate| candidate.group_id.as_ref() == Some(group_id));
    candidates.sort_by(compare_tab_candidates);
    candidates.first().map(|candidate| candidate.node_id)
}

fn filter_modal_target(
    tree: &UiTree,
    target: Option<UiNodeId>,
    modal_group: Option<&UiNavigationGroupId>,
) -> Option<UiNodeId> {
    let Some(modal_group) = modal_group else {
        return target;
    };
    target.filter(|target| modal_group_for(tree, *target).as_ref() == Some(modal_group))
}

#[derive(Clone)]
struct NavigationCandidate {
    node_id: UiNodeId,
    tab_order: i32,
    tabbable: bool,
    group_order: i32,
    group_id: Option<UiNavigationGroupId>,
    modal: bool,
    paint_order: u64,
}

fn target_for_direction(
    directional: &UiDirectionalNavigation,
    kind: UiNavigationEventKind,
) -> &UiDirectionalNavigationTarget {
    match kind {
        UiNavigationEventKind::Up => &directional.up,
        UiNavigationEventKind::Down => &directional.down,
        UiNavigationEventKind::Left => &directional.left,
        UiNavigationEventKind::Right => &directional.right,
        UiNavigationEventKind::Activate
        | UiNavigationEventKind::Cancel
        | UiNavigationEventKind::Home
        | UiNavigationEventKind::End
        | UiNavigationEventKind::Next
        | UiNavigationEventKind::Previous => &UiDirectionalNavigationTarget::Auto,
    }
}

fn compare_tab_candidates(left: &NavigationCandidate, right: &NavigationCandidate) -> Ordering {
    left.group_order
        .cmp(&right.group_order)
        .then_with(|| left.tab_order.cmp(&right.tab_order))
        .then_with(|| left.paint_order.cmp(&right.paint_order))
        .then_with(|| left.node_id.cmp(&right.node_id))
}

fn compare_tree_candidates(left: &NavigationCandidate, right: &NavigationCandidate) -> Ordering {
    left.paint_order
        .cmp(&right.paint_order)
        .then_with(|| left.node_id.cmp(&right.node_id))
}

fn collect_focusable_nodes(
    tree: &UiTree,
    node_id: UiNodeId,
    focusable: &mut Vec<UiNodeId>,
) -> Result<(), UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if node.is_focus_candidate() {
        focusable.push(node_id);
    }
    for child_id in &node.children {
        collect_focusable_nodes(tree, *child_id, focusable)?;
    }
    Ok(())
}

fn nearest_focusable_in_direction(
    tree: &UiTree,
    current: UiNodeId,
    kind: UiNavigationEventKind,
    focusable: &[UiNodeId],
) -> Result<Option<UiNodeId>, UiTreeError> {
    const DIRECTION_EPSILON: f32 = 0.0001;

    let origin = tree
        .nodes
        .get(&current)
        .ok_or(UiTreeError::MissingNode(current))?
        .layout_cache
        .frame
        .center();
    let mut best: Option<(UiNodeId, f32, f32)> = None;

    for candidate in focusable {
        if *candidate == current {
            continue;
        }
        let center = tree
            .nodes
            .get(candidate)
            .ok_or(UiTreeError::MissingNode(*candidate))?
            .layout_cache
            .frame
            .center();
        let delta_x = center.x - origin.x;
        let delta_y = center.y - origin.y;
        let (primary, lateral) = match kind {
            UiNavigationEventKind::Right => (delta_x, delta_y.abs()),
            UiNavigationEventKind::Left => (-delta_x, delta_y.abs()),
            UiNavigationEventKind::Down => (delta_y, delta_x.abs()),
            UiNavigationEventKind::Up => (-delta_y, delta_x.abs()),
            UiNavigationEventKind::Activate
            | UiNavigationEventKind::Cancel
            | UiNavigationEventKind::Home
            | UiNavigationEventKind::End
            | UiNavigationEventKind::Next
            | UiNavigationEventKind::Previous => continue,
        };
        if primary <= DIRECTION_EPSILON {
            continue;
        }

        let slope = lateral / primary;
        let distance_sq = delta_x * delta_x + delta_y * delta_y;
        let is_better = best
            .as_ref()
            .is_none_or(|(_, best_slope, best_distance_sq)| {
                slope < *best_slope - DIRECTION_EPSILON
                    || ((slope - *best_slope).abs() <= DIRECTION_EPSILON
                        && distance_sq < *best_distance_sq)
            });
        if is_better {
            best = Some((*candidate, slope, distance_sq));
        }
    }

    Ok(best.map(|(candidate, _, _)| candidate))
}

fn nearest_navigation_candidate_in_direction(
    tree: &UiTree,
    current: UiNodeId,
    kind: UiNavigationEventKind,
    focusable: &[NavigationCandidate],
) -> Result<Option<UiNodeId>, UiTreeError> {
    const DIRECTION_EPSILON: f32 = 0.0001;

    let origin = tree
        .nodes
        .get(&current)
        .ok_or(UiTreeError::MissingNode(current))?
        .layout_cache
        .frame
        .center();
    let mut best: Option<(UiNodeId, f32, f32)> = None;

    for candidate in focusable {
        if candidate.node_id == current {
            continue;
        }
        let center = tree
            .nodes
            .get(&candidate.node_id)
            .ok_or(UiTreeError::MissingNode(candidate.node_id))?
            .layout_cache
            .frame
            .center();
        let delta_x = center.x - origin.x;
        let delta_y = center.y - origin.y;
        let (primary, lateral) = match kind {
            UiNavigationEventKind::Right => (delta_x, delta_y.abs()),
            UiNavigationEventKind::Left => (-delta_x, delta_y.abs()),
            UiNavigationEventKind::Down => (delta_y, delta_x.abs()),
            UiNavigationEventKind::Up => (-delta_y, delta_x.abs()),
            UiNavigationEventKind::Activate
            | UiNavigationEventKind::Cancel
            | UiNavigationEventKind::Home
            | UiNavigationEventKind::End
            | UiNavigationEventKind::Next
            | UiNavigationEventKind::Previous => continue,
        };
        if primary <= DIRECTION_EPSILON {
            continue;
        }

        let slope = lateral / primary;
        let distance_sq = delta_x * delta_x + delta_y * delta_y;
        let is_better = best
            .as_ref()
            .is_none_or(|(_, best_slope, best_distance_sq)| {
                slope < *best_slope - DIRECTION_EPSILON
                    || ((slope - *best_slope).abs() <= DIRECTION_EPSILON
                        && distance_sq < *best_distance_sq)
            });
        if is_better {
            best = Some((candidate.node_id, slope, distance_sq));
        }
    }

    Ok(best.map(|(candidate, _, _)| candidate))
}
