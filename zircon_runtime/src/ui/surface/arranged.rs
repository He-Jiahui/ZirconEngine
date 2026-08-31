use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSlotKind},
    surface::{
        UiArrangedNode, UiArrangedSlotSummary, UiArrangedTree, UiCanvasLayerGroup, UiFocusPath,
        UiPersistentSequenceCowStats,
    },
    tree::{UiInputPolicy, UiTree, UiTreeError},
};

pub fn build_arranged_tree(tree: &UiTree) -> UiArrangedTree {
    let slot_indices = arranged_slot_indices(tree);
    let mut nodes: Vec<_> = tree
        .nodes
        .values()
        .map(|node| arranged_node_from_tree(tree, node.node_id, &slot_indices))
        .collect();
    nodes.sort_by_key(|node| (node.z_index, node.paint_order, node.node_id));
    let draw_order = nodes.iter().map(|node| node.node_id).collect::<Vec<_>>();
    let canvas_layers = arranged_canvas_layers(tree);
    let mut arranged_tree = UiArrangedTree {
        tree_id: tree.tree_id.clone(),
        roots: tree.roots.clone().into(),
        nodes: nodes.into(),
        draw_order: draw_order.into(),
        canvas_layers: canvas_layers.into(),
        ..UiArrangedTree::default()
    };
    arranged_tree
}

pub(crate) fn patch_arranged_tree_geometry(
    tree: &UiTree,
    arranged_tree: &mut UiArrangedTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
    checked_node_ids: &BTreeSet<UiNodeId>,
    node_indices: &BTreeMap<UiNodeId, usize>,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> Option<BTreeSet<UiNodeId>> {
    if checked_node_ids.is_empty()
        || !changed_node_ids.is_subset(checked_node_ids)
        || arranged_tree.tree_id != tree.tree_id
        || arranged_tree.nodes.len() != tree.nodes.len()
        || arranged_tree.roots != tree.roots
        || node_indices.len() != arranged_tree.nodes.len()
        || slot_indices.len() != tree.layout_slots().len()
    {
        return None;
    }

    let mut affected_node_ids = changed_node_ids.clone();
    for node_id in changed_node_ids {
        let node = tree.node(*node_id)?;
        if node.clip_to_bounds || node.container.clips_to_bounds() {
            collect_tree_descendants(tree, *node_id, &mut affected_node_ids)?;
        }
    }
    if !affected_node_ids.is_subset(checked_node_ids) {
        return None;
    }

    let mut replacements = Vec::with_capacity(affected_node_ids.len());
    for node_id in checked_node_ids {
        let node = tree.node(*node_id)?;
        let previous_index = node_indices.get(node_id).copied()?;
        let previous = arranged_tree.nodes.get(previous_index)?;
        let next_frame = node.layout_cache.frame;
        let next_clip_frame =
            effective_node_clip_frame(tree, node.node_id).unwrap_or(node.layout_cache.frame);
        let next_slot = arranged_node_slot_indexed(tree, node.node_id, slot_indices);
        if !same_tree_non_geometry_fields(tree, previous, node, slot_indices) {
            return None;
        }
        if affected_node_ids.contains(node_id) {
            if !same_slot_non_geometry_fields(previous.slot.as_ref(), next_slot.as_ref()) {
                return None;
            }
            replacements.push((previous_index, next_frame, next_clip_frame, next_slot));
        } else if previous.frame != next_frame
            || previous.clip_frame != next_clip_frame
            || previous.slot != next_slot
        {
            return None;
        }
    }

    let mut cow_stats = UiPersistentSequenceCowStats::default();
    for (index, frame, clip_frame, slot) in replacements {
        let (current, current_stats) = arranged_tree.nodes.get_mut_with_stats(index)?;
        cow_stats.accumulate(current_stats);
        current.frame = frame;
        current.clip_frame = clip_frame;
        current.slot = slot;
    }
    record_arranged_persistent_cow(cow_stats);
    Some(affected_node_ids)
}

pub(crate) fn authored_geometry_affected_node_ids(
    tree: &UiTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
) -> Option<BTreeSet<UiNodeId>> {
    if changed_node_ids.is_empty() {
        return Some(BTreeSet::new());
    }

    let mut affected_node_ids = changed_node_ids.clone();
    for node_id in changed_node_ids {
        let node = tree.node(*node_id)?;
        if node.clip_to_bounds || node.container.clips_to_bounds() {
            collect_tree_descendants(tree, *node_id, &mut affected_node_ids)?;
        }
    }
    Some(affected_node_ids)
}

pub(crate) fn patch_arranged_tree_input(
    tree: &UiTree,
    arranged_tree: &mut UiArrangedTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
    pending_geometry_node_ids: &BTreeSet<UiNodeId>,
    node_indices: &BTreeMap<UiNodeId, usize>,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> Option<BTreeSet<UiNodeId>> {
    if changed_node_ids.is_empty()
        || arranged_tree.tree_id != tree.tree_id
        || arranged_tree.nodes.len() != tree.nodes.len()
        || arranged_tree.roots != tree.roots
        || node_indices.len() != arranged_tree.nodes.len()
        || slot_indices.len() != tree.layout_slots().len()
    {
        return None;
    }

    let mut affected_node_ids = changed_node_ids.clone();
    for node_id in changed_node_ids {
        let tree_node = tree.node(*node_id)?;
        let previous = arranged_node_indexed(arranged_tree, node_indices, *node_id).ok()?;
        if previous.visibility != tree_node.effective_visibility() {
            // Visibility also changes render ancestry and canvas-layer membership.
            return None;
        }
        if previous.input_policy != tree_node.input_policy
            || previous.pointer_events != tree_node.pointer_events
        {
            collect_tree_descendants(tree, *node_id, &mut affected_node_ids)?;
        }
    }

    let mut cow_stats = UiPersistentSequenceCowStats::default();
    for node_id in &affected_node_ids {
        let previous_index = node_indices.get(node_id).copied()?;
        let previous = arranged_tree.nodes.get(previous_index)?;
        let tree_node = tree.node(*node_id)?;
        if !same_arranged_input_patch_structure(
            tree,
            previous,
            tree_node,
            pending_geometry_node_ids.contains(node_id),
            slot_indices,
        ) {
            return None;
        }
    }

    for node_id in &affected_node_ids {
        let previous_index = *node_indices
            .get(node_id)
            .expect("input patch node index was validated");
        let tree_node = tree
            .node(*node_id)
            .expect("input patch tree node was validated");
        let (current, current_stats) = arranged_tree
            .nodes
            .get_mut_with_stats(previous_index)
            .expect("input patch arranged node was validated");
        cow_stats.accumulate(current_stats);
        let next_control_id = tree_node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref());
        let next_focusable = tree_node.is_focus_candidate();
        let control_id_changed = current.control_id.as_deref() != next_control_id;
        if changed_node_ids.contains(node_id)
            || current.input_policy != tree_node.input_policy
            || current.pointer_events != tree_node.pointer_events
            || current.enabled != tree_node.state_flags.enabled
            || current.clickable != tree_node.state_flags.clickable
            || current.hoverable != tree_node.state_flags.hoverable
            || current.focusable != next_focusable
            || control_id_changed
        {
            current.input_policy = tree_node.input_policy;
            current.pointer_events = tree_node.pointer_events;
            current.enabled = tree_node.state_flags.enabled;
            current.clickable = tree_node.state_flags.clickable;
            current.hoverable = tree_node.state_flags.hoverable;
            current.focusable = next_focusable;
            if control_id_changed {
                current.control_id = next_control_id.map(str::to_owned);
            }
        }
    }
    record_arranged_persistent_cow(cow_stats);
    Some(affected_node_ids)
}

fn record_arranged_persistent_cow(stats: UiPersistentSequenceCowStats) {
    crate::profile_counter!(
        "runtime",
        "ui.arranged.persistent_cow_item_clone_count",
        stats.cloned_item_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.arranged.persistent_cow_segment_clone_count",
        stats.cloned_segment_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.arranged.persistent_cow_directory_node_clone_count",
        stats.cloned_directory_node_count
    );
}

fn collect_tree_descendants(
    tree: &UiTree,
    root_id: UiNodeId,
    node_ids: &mut BTreeSet<UiNodeId>,
) -> Option<()> {
    let mut pending = tree.node(root_id)?.children.clone();
    while let Some(node_id) = pending.pop() {
        if !node_ids.insert(node_id) {
            continue;
        }
        pending.extend(tree.node(node_id)?.children.iter().copied());
    }
    Some(())
}

fn same_arranged_input_patch_structure(
    tree: &UiTree,
    previous: &UiArrangedNode,
    tree_node: &zircon_runtime_interface::ui::tree::UiTreeNode,
    pending_geometry_change: bool,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> bool {
    previous.node_id == tree_node.node_id
        && previous.node_path == tree_node.node_path
        && previous.parent == tree_node.parent
        && previous.children == tree_node.children
        && (pending_geometry_change
            || (previous.frame == tree_node.layout_cache.frame
                && previous.clip_frame
                    == effective_node_clip_frame(tree, tree_node.node_id)
                        .unwrap_or(tree_node.layout_cache.frame)))
        && previous.z_index == arranged_node_z_index_indexed(tree, tree_node.node_id, slot_indices)
        && previous.paint_order == tree_node.paint_order
        && previous.visibility == tree_node.effective_visibility()
        && previous.clip_to_bounds
            == (tree_node.clip_to_bounds || tree_node.container.clips_to_bounds())
        && (pending_geometry_change
            || previous.slot == arranged_node_slot_indexed(tree, tree_node.node_id, slot_indices))
}

pub(crate) fn arranged_node_indices(arranged_tree: &UiArrangedTree) -> BTreeMap<UiNodeId, usize> {
    arranged_tree
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect()
}

pub(crate) fn arranged_node_indexed<'a>(
    arranged_tree: &'a UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<&'a UiArrangedNode, UiTreeError> {
    let Some(index) = node_indices.get(&node_id).copied() else {
        return Err(UiTreeError::MissingNode(node_id));
    };
    arranged_tree
        .nodes
        .get(index)
        .filter(|node| node.node_id == node_id)
        .ok_or(UiTreeError::MissingNode(node_id))
}

pub(crate) fn arranged_bubble_route_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut route = Vec::new();
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_node_indexed(arranged_tree, node_indices, id)?;
        route.push(id);
        current = node.parent;
    }
    Ok(route)
}

pub(crate) fn is_arranged_render_visible_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<bool, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_node_indexed(arranged_tree, node_indices, id)?;
        if !node.is_render_visible() {
            return Ok(false);
        }
        current = node.parent;
    }
    Ok(true)
}

pub(crate) fn is_arranged_child_hit_path_visible_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<bool, UiTreeError> {
    let node = arranged_node_indexed(arranged_tree, node_indices, node_id)?;
    if !node.allows_self_pointer_hit_test() {
        return Ok(false);
    }
    let mut current = node.parent;
    while let Some(id) = current {
        let ancestor = arranged_node_indexed(arranged_tree, node_indices, id)?;
        if !ancestor.allows_child_pointer_hit_test() {
            return Ok(false);
        }
        current = ancestor.parent;
    }
    Ok(true)
}

pub(crate) fn arranged_effective_input_policy_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<UiInputPolicy, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_node_indexed(arranged_tree, node_indices, id)?;
        match node.input_policy {
            UiInputPolicy::Inherit => current = node.parent,
            explicit => return Ok(explicit),
        }
    }
    Ok(UiInputPolicy::Receive)
}

pub(crate) fn arranged_slot_indices(tree: &UiTree) -> BTreeMap<UiNodeId, usize> {
    let mut indices = BTreeMap::new();
    for (index, slot) in tree.layout_slots().iter().enumerate() {
        indices.entry(slot.child_id).or_insert(index);
    }
    indices
}

fn arranged_node_from_tree(
    tree: &UiTree,
    node_id: UiNodeId,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> UiArrangedNode {
    let node = tree
        .node(node_id)
        .expect("arranged node source must exist in the UI tree");
    UiArrangedNode {
        node_id: node.node_id,
        node_path: node.node_path.clone(),
        parent: node.parent,
        children: node.children.clone(),
        frame: node.layout_cache.frame,
        clip_frame: effective_node_clip_frame(tree, node.node_id)
            .unwrap_or(node.layout_cache.frame),
        z_index: arranged_node_z_index_indexed(tree, node.node_id, slot_indices),
        paint_order: node.paint_order,
        visibility: node.effective_visibility(),
        input_policy: node.input_policy,
        pointer_events: node.pointer_events,
        enabled: node.state_flags.enabled,
        clickable: node.state_flags.clickable,
        hoverable: node.state_flags.hoverable,
        focusable: node.is_focus_candidate(),
        clip_to_bounds: node.clip_to_bounds || node.container.clips_to_bounds(),
        control_id: node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.clone()),
        slot: arranged_node_slot_indexed(tree, node.node_id, slot_indices),
    }
}

fn same_tree_non_geometry_fields(
    tree: &UiTree,
    arranged: &UiArrangedNode,
    tree_node: &zircon_runtime_interface::ui::tree::UiTreeNode,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> bool {
    arranged.node_id == tree_node.node_id
        && arranged.node_path == tree_node.node_path
        && arranged.parent == tree_node.parent
        && arranged.children == tree_node.children
        && arranged.z_index == arranged_node_z_index_indexed(tree, tree_node.node_id, slot_indices)
        && arranged.paint_order == tree_node.paint_order
        && arranged.visibility == tree_node.effective_visibility()
        && arranged.input_policy == tree_node.input_policy
        && arranged.pointer_events == tree_node.pointer_events
        && arranged.enabled == tree_node.state_flags.enabled
        && arranged.clickable == tree_node.state_flags.clickable
        && arranged.hoverable == tree_node.state_flags.hoverable
        && arranged.focusable == tree_node.is_focus_candidate()
        && arranged.clip_to_bounds
            == (tree_node.clip_to_bounds || tree_node.container.clips_to_bounds())
        && arranged.control_id.as_deref()
            == tree_node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
}

fn same_slot_non_geometry_fields(
    previous: Option<&UiArrangedSlotSummary>,
    next: Option<&UiArrangedSlotSummary>,
) -> bool {
    match (previous, next) {
        (None, None) => true,
        (Some(previous), Some(next)) => {
            previous.parent_id == next.parent_id
                && previous.child_id == next.child_id
                && previous.kind == next.kind
                && previous.order == next.order
                && previous.z_order == next.z_order
        }
        _ => false,
    }
}

fn arranged_node_slot_indexed(
    tree: &UiTree,
    child_id: UiNodeId,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> Option<zircon_runtime_interface::ui::surface::UiArrangedSlotSummary> {
    let child = tree.node(child_id)?;
    let parent_id = child.parent?;
    let slot_kind = tree.node(parent_id)?.container.child_slot_kind()?;
    let slot = tree.layout_slot(*slot_indices.get(&child_id)?)?;
    (slot.parent_id == parent_id && slot.child_id == child_id && slot.kind == slot_kind)
        .then(|| slot.into())
}

fn arranged_node_z_index_indexed(
    tree: &UiTree,
    child_id: UiNodeId,
    slot_indices: &BTreeMap<UiNodeId, usize>,
) -> i32 {
    let Some(node) = tree.node(child_id) else {
        return 0;
    };
    let slot_z = slot_indices
        .get(&child_id)
        .and_then(|index| tree.layout_slot(*index))
        .filter(|slot| {
            slot.child_id == child_id
                && matches!(slot.kind, UiSlotKind::Overlay | UiSlotKind::Canvas)
        })
        .map(|slot| slot.z_order)
        .unwrap_or_default();
    node.z_index.saturating_add(slot_z)
}

fn arranged_canvas_layers(tree: &UiTree) -> Vec<UiCanvasLayerGroup> {
    let mut layers = Vec::new();
    let mut canvas_slots_by_parent = BTreeMap::<UiNodeId, Vec<_>>::new();
    for slot in tree
        .layout_slots()
        .iter()
        .filter(|slot| slot.kind == UiSlotKind::Canvas)
    {
        canvas_slots_by_parent
            .entry(slot.parent_id)
            .or_default()
            .push(slot);
    }
    let mut canvas_parents: Vec<_> = tree
        .nodes
        .values()
        .filter(|node| node.container.child_slot_kind() == Some(UiSlotKind::Canvas))
        .collect();
    canvas_parents.sort_by_key(|node| node.node_id);

    for parent in canvas_parents {
        let child_ids = parent.children.iter().copied().collect::<BTreeSet<_>>();
        let mut children: Vec<_> = canvas_slots_by_parent
            .get(&parent.node_id)
            .into_iter()
            .flatten()
            .copied()
            .filter(|slot| child_ids.contains(&slot.child_id))
            .filter(|slot| is_tree_render_visible(tree, slot.child_id))
            .collect();
        children.sort_by_key(|slot| {
            let paint_order = tree
                .node(slot.child_id)
                .map(|node| node.paint_order)
                .unwrap_or_default();
            (slot.z_order, paint_order, slot.child_id)
        });

        let mut active_z_order: Option<i32> = None;
        let mut next_layer_index = 0u32;
        for slot in children {
            if active_z_order != Some(slot.z_order) {
                active_z_order = Some(slot.z_order);
                layers.push(UiCanvasLayerGroup {
                    parent_id: parent.node_id,
                    layer_index: next_layer_index,
                    z_order: slot.z_order,
                    child_ids: Vec::new(),
                });
                next_layer_index = next_layer_index.saturating_add(1);
            }
            let layer = layers
                .last_mut()
                .expect("Canvas layer should exist before adding a child");
            layer.child_ids.push(slot.child_id);
        }
    }
    layers
}

fn is_tree_render_visible(tree: &UiTree, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = tree.node(id) else {
            return false;
        };
        if !node.effective_visibility().is_render_visible() {
            return false;
        }
        current = node.parent;
    }
    true
}

pub fn arranged_bubble_route(
    arranged_tree: &UiArrangedTree,
    node_id: UiNodeId,
) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut route = Vec::new();
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_tree.get(id).ok_or(UiTreeError::MissingNode(id))?;
        route.push(id);
        current = node.parent;
    }
    Ok(route)
}

pub fn arranged_focus_path(
    arranged_tree: &UiArrangedTree,
    focused: Option<UiNodeId>,
) -> UiFocusPath {
    let Some(focused) = focused else {
        return UiFocusPath::default();
    };
    arranged_bubble_route(arranged_tree, focused)
        .map(|route| UiFocusPath::from_bubble_route(Some(focused), route))
        .unwrap_or_else(|_| UiFocusPath {
            focused: Some(focused),
            ..UiFocusPath::default()
        })
}

pub(crate) fn arranged_focus_path_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    focused: Option<UiNodeId>,
) -> UiFocusPath {
    let Some(focused) = focused else {
        return UiFocusPath::default();
    };
    arranged_bubble_route_indexed(arranged_tree, node_indices, focused)
        .map(|route| UiFocusPath::from_bubble_route(Some(focused), route))
        .unwrap_or_else(|_| UiFocusPath {
            focused: Some(focused),
            ..UiFocusPath::default()
        })
}

pub(crate) fn arranged_focus_path_matches_indexed(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    path: &UiFocusPath,
    focused: Option<UiNodeId>,
) -> bool {
    if path.focused != focused || path.root_to_leaf.len() != path.bubble_route.len() {
        return false;
    }
    let Some(focused) = focused else {
        return path.root_to_leaf.is_empty() && path.bubble_route.is_empty();
    };
    if path.bubble_route.is_empty() {
        return path.root_to_leaf.is_empty()
            && arranged_node_indexed(arranged_tree, node_indices, focused).is_err();
    }
    if path.bubble_route.first().copied() != Some(focused)
        || !path.root_to_leaf.iter().rev().eq(path.bubble_route.iter())
    {
        return false;
    }
    path.bubble_route
        .iter()
        .enumerate()
        .all(|(route_index, node_id)| {
            let Ok(node) = arranged_node_indexed(arranged_tree, node_indices, *node_id) else {
                return false;
            };
            node.parent == path.bubble_route.get(route_index + 1).copied()
        })
}

pub fn is_arranged_render_visible(
    arranged_tree: &UiArrangedTree,
    node_id: UiNodeId,
) -> Result<bool, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_tree.get(id).ok_or(UiTreeError::MissingNode(id))?;
        if !node.is_render_visible() {
            return Ok(false);
        }
        current = node.parent;
    }
    Ok(true)
}

pub fn is_arranged_child_hit_path_visible(
    arranged_tree: &UiArrangedTree,
    node_id: UiNodeId,
) -> Result<bool, UiTreeError> {
    let Some(node) = arranged_tree.get(node_id) else {
        return Err(UiTreeError::MissingNode(node_id));
    };
    if !node.allows_self_pointer_hit_test() {
        return Ok(false);
    }

    let mut current = node.parent;
    while let Some(id) = current {
        let ancestor = arranged_tree.get(id).ok_or(UiTreeError::MissingNode(id))?;
        if !ancestor.allows_child_pointer_hit_test() {
            return Ok(false);
        }
        current = ancestor.parent;
    }
    Ok(true)
}

pub fn arranged_effective_input_policy(
    arranged_tree: &UiArrangedTree,
    node_id: UiNodeId,
) -> Result<UiInputPolicy, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_tree.get(id).ok_or(UiTreeError::MissingNode(id))?;
        match node.input_policy {
            UiInputPolicy::Inherit => current = node.parent,
            explicit => return Ok(explicit),
        }
    }
    Ok(UiInputPolicy::Receive)
}

fn effective_node_clip_frame(tree: &UiTree, node_id: UiNodeId) -> Option<UiFrame> {
    let node = tree.node(node_id)?;
    let mut clip = node_clip_frame(node);
    let mut current = node.parent;
    while let Some(id) = current {
        let ancestor = tree.node(id)?;
        let ancestor_clip = node_clip_frame(ancestor);
        clip = match (clip, ancestor_clip) {
            (Some(a), Some(b)) => a.intersection(b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        current = ancestor.parent;
    }
    clip
}

fn node_clip_frame(node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> Option<UiFrame> {
    if node.clip_to_bounds || node.container.clips_to_bounds() {
        Some(
            node.layout_cache
                .clip_frame
                .unwrap_or(node.layout_cache.frame),
        )
    } else {
        node.layout_cache.clip_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        tree::{UiPointerEvents, UiTreeNode},
    };

    #[test]
    fn indexed_focus_path_matches_legacy_route_and_missing_node_fallback() {
        let root_id = UiNodeId::new(1);
        let child_id = UiNodeId::new(2);
        let leaf_id = UiNodeId::new(3);
        let missing_id = UiNodeId::new(4);
        let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
        let mut tree = UiTree::new(UiTreeId::new("ui.arranged.focus-path-index"));
        tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame));
        tree.insert_child(
            root_id,
            UiTreeNode::new(child_id, UiNodePath::new("root/child")).with_frame(frame),
        )
        .unwrap();
        tree.insert_child(
            child_id,
            UiTreeNode::new(leaf_id, UiNodePath::new("root/child/leaf")).with_frame(frame),
        )
        .unwrap();

        let arranged_tree = build_arranged_tree(&tree);
        let node_indices = arranged_node_indices(&arranged_tree);
        assert_eq!(
            arranged_focus_path_indexed(&arranged_tree, &node_indices, Some(leaf_id)),
            arranged_focus_path(&arranged_tree, Some(leaf_id))
        );

        let missing = arranged_focus_path_indexed(&arranged_tree, &node_indices, Some(missing_id));
        assert_eq!(missing.focused, Some(missing_id));
        assert!(missing.bubble_route.is_empty());
    }

    #[test]
    fn indexed_focus_path_validation_rejects_a_reparented_route() {
        let root_id = UiNodeId::new(1);
        let left_id = UiNodeId::new(2);
        let right_id = UiNodeId::new(3);
        let leaf_id = UiNodeId::new(4);
        let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
        let build_tree = |leaf_parent| {
            let mut tree = UiTree::new(UiTreeId::new("ui.arranged.focus-path-reparent"));
            tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame));
            tree.insert_child(
                root_id,
                UiTreeNode::new(left_id, UiNodePath::new("root/left")).with_frame(frame),
            )
            .unwrap();
            tree.insert_child(
                root_id,
                UiTreeNode::new(right_id, UiNodePath::new("root/right")).with_frame(frame),
            )
            .unwrap();
            tree.insert_child(
                leaf_parent,
                UiTreeNode::new(leaf_id, UiNodePath::new("root/leaf")).with_frame(frame),
            )
            .unwrap();
            tree
        };

        let before = build_tree(left_id);
        let before_arranged = build_arranged_tree(&before);
        let before_indices = arranged_node_indices(&before_arranged);
        let path = arranged_focus_path_indexed(&before_arranged, &before_indices, Some(leaf_id));
        assert!(arranged_focus_path_matches_indexed(
            &before_arranged,
            &before_indices,
            &path,
            Some(leaf_id),
        ));

        let after = build_tree(right_id);
        let after_arranged = build_arranged_tree(&after);
        let after_indices = arranged_node_indices(&after_arranged);
        assert!(!arranged_focus_path_matches_indexed(
            &after_arranged,
            &after_indices,
            &path,
            Some(leaf_id),
        ));
        assert_eq!(
            arranged_focus_path_indexed(&after_arranged, &after_indices, Some(leaf_id))
                .bubble_route,
            vec![leaf_id, right_id, root_id]
        );
    }

    #[test]
    fn parent_input_patch_preserves_arranged_structure_allocations_for_descendants() {
        let root_id = UiNodeId::new(1);
        let child_id = UiNodeId::new(2);
        let leaf_id = UiNodeId::new(3);
        let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
        let mut tree = UiTree::new(UiTreeId::new("ui.arranged.input-patch"));
        let mut root = UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(frame);
        root.input_policy = UiInputPolicy::Receive;
        tree.insert_root(root);
        tree.insert_child(
            root_id,
            UiTreeNode::new(child_id, UiNodePath::new("root/child")).with_frame(frame),
        )
        .unwrap();
        tree.insert_child(
            child_id,
            UiTreeNode::new(leaf_id, UiNodePath::new("root/child/leaf")).with_frame(frame),
        )
        .unwrap();

        let mut arranged_tree = build_arranged_tree(&tree);
        let node_indices = arranged_node_indices(&arranged_tree);
        let slot_indices = arranged_slot_indices(&tree);
        let root_index = node_indices[&root_id];
        let child_index = node_indices[&child_id];
        let root_path_ptr = arranged_tree.nodes[root_index].node_path.0.as_ptr();
        let root_children_ptr = arranged_tree.nodes[root_index].children.as_ptr();
        let child_path_ptr = arranged_tree.nodes[child_index].node_path.0.as_ptr();
        let child_children_ptr = arranged_tree.nodes[child_index].children.as_ptr();

        tree.node_mut(root_id).unwrap().input_policy = UiInputPolicy::Ignore;
        tree.node_mut(root_id).unwrap().pointer_events = UiPointerEvents::None;
        tree.node_mut(child_id).unwrap().state_flags.clickable = true;
        let affected = patch_arranged_tree_input(
            &tree,
            &mut arranged_tree,
            &BTreeSet::from([root_id]),
            &BTreeSet::new(),
            &node_indices,
            &slot_indices,
        )
        .expect("input-only mutation should stay on the incremental path");

        assert_eq!(affected, BTreeSet::from([root_id, child_id, leaf_id]));
        assert_eq!(
            arranged_tree.nodes[root_index].node_path.0.as_ptr(),
            root_path_ptr
        );
        assert_eq!(
            arranged_tree.nodes[root_index].children.as_ptr(),
            root_children_ptr
        );
        assert_eq!(
            arranged_tree.nodes[child_index].node_path.0.as_ptr(),
            child_path_ptr
        );
        assert_eq!(
            arranged_tree.nodes[child_index].children.as_ptr(),
            child_children_ptr
        );
        assert_eq!(
            arranged_tree.nodes[root_index].input_policy,
            UiInputPolicy::Ignore
        );
        assert!(arranged_tree.nodes[child_index].clickable);
        assert_eq!(
            arranged_effective_input_policy_indexed(&arranged_tree, &node_indices, leaf_id,)
                .unwrap(),
            UiInputPolicy::Ignore
        );
        assert!(!is_arranged_child_hit_path_visible_indexed(
            &arranged_tree,
            &node_indices,
            leaf_id,
        )
        .unwrap());
    }

    #[test]
    fn clipping_geometry_patch_returns_the_exact_affected_subtree() {
        let root_id = UiNodeId::new(1);
        let child_id = UiNodeId::new(2);
        let sibling_id = UiNodeId::new(3);
        let mut tree = UiTree::new(UiTreeId::new("ui.arranged.clipping-geometry-patch"));
        let mut root = UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0));
        root.clip_to_bounds = true;
        tree.insert_root(root);
        tree.insert_child(
            root_id,
            UiTreeNode::new(child_id, UiNodePath::new("root/child"))
                .with_frame(UiFrame::new(80.0, 0.0, 40.0, 20.0)),
        )
        .unwrap();
        tree.insert_root(
            UiTreeNode::new(sibling_id, UiNodePath::new("sibling"))
                .with_frame(UiFrame::new(200.0, 0.0, 20.0, 20.0)),
        );

        let mut arranged_tree = build_arranged_tree(&tree);
        let node_indices = arranged_node_indices(&arranged_tree);
        let slot_indices = arranged_slot_indices(&tree);
        tree.node_mut(root_id).unwrap().layout_cache.frame = UiFrame::new(0.0, 0.0, 90.0, 100.0);

        let affected = patch_arranged_tree_geometry(
            &tree,
            &mut arranged_tree,
            &BTreeSet::from([root_id]),
            &BTreeSet::from([root_id, child_id]),
            &node_indices,
            &slot_indices,
        )
        .expect("clipping geometry should patch its retained subtree");

        assert_eq!(affected, BTreeSet::from([root_id, child_id]));
        let child = arranged_node_indexed(&arranged_tree, &node_indices, child_id).unwrap();
        assert_eq!(child.clip_frame, UiFrame::new(0.0, 0.0, 90.0, 100.0));
        assert_eq!(
            child.frame.intersection(child.clip_frame),
            Some(UiFrame::new(80.0, 0.0, 10.0, 20.0))
        );
        assert_eq!(
            arranged_node_indexed(&arranged_tree, &node_indices, sibling_id)
                .unwrap()
                .clip_frame,
            UiFrame::new(200.0, 0.0, 20.0, 20.0)
        );
    }

    #[test]
    fn input_patch_allows_geometry_committed_by_the_following_patch() {
        let root_id = UiNodeId::new(1);
        let original_frame = UiFrame::new(0.0, 0.0, 100.0, 60.0);
        let resized_frame = UiFrame::new(0.0, 0.0, 140.0, 80.0);
        let mut tree = UiTree::new(UiTreeId::new("ui.arranged.input-geometry-patch"));
        tree.insert_root(
            UiTreeNode::new(root_id, UiNodePath::new("root")).with_frame(original_frame),
        );
        let mut arranged_tree = build_arranged_tree(&tree);
        let node_indices = arranged_node_indices(&arranged_tree);
        let slot_indices = arranged_slot_indices(&tree);
        let root = tree.node_mut(root_id).expect("root should exist");
        root.layout_cache.frame = resized_frame;
        root.state_flags.enabled = false;

        let input_affected = patch_arranged_tree_input(
            &tree,
            &mut arranged_tree,
            &BTreeSet::from([root_id]),
            &BTreeSet::from([root_id]),
            &node_indices,
            &slot_indices,
        )
        .expect("input patch should admit geometry owned by the following patch");
        let input_patched = arranged_node_indexed(&arranged_tree, &node_indices, root_id).unwrap();
        assert!(!input_patched.enabled);
        assert_eq!(input_patched.frame, original_frame);

        let geometry_affected = patch_arranged_tree_geometry(
            &tree,
            &mut arranged_tree,
            &BTreeSet::from([root_id]),
            &BTreeSet::from([root_id]),
            &node_indices,
            &slot_indices,
        )
        .expect("geometry patch should validate the combined transaction");

        assert_eq!(input_affected, BTreeSet::from([root_id]));
        assert_eq!(geometry_affected, BTreeSet::from([root_id]));
        let committed = arranged_node_indexed(&arranged_tree, &node_indices, root_id).unwrap();
        assert!(!committed.enabled);
        assert_eq!(committed.frame, resized_frame);
    }
}
