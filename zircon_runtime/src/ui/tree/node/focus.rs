use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
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
                        | UiNavigationEventKind::Next
                        | UiNavigationEventKind::Previous => None,
                    });
                };
                nearest_focusable_in_direction(self, current, kind, &focusable)
            }
            UiNavigationEventKind::Activate | UiNavigationEventKind::Cancel => Ok(None),
        }
    }
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
