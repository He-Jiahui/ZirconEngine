use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSlotKind},
    surface::{
        UiArrangedNode, UiArrangedSlotSummary, UiArrangedTree, UiCanvasLayerGroup, UiFocusPath,
    },
    tree::{UiInputPolicy, UiTree, UiTreeError},
};

pub fn build_arranged_tree(tree: &UiTree) -> UiArrangedTree {
    let mut nodes: Vec<_> = tree
        .nodes
        .values()
        .map(|node| arranged_node_from_tree(tree, node.node_id))
        .collect();
    nodes.sort_by_key(|node| (node.z_index, node.paint_order, node.node_id));
    let draw_order = nodes.iter().map(|node| node.node_id).collect();
    let canvas_layers = arranged_canvas_layers(tree);
    let mut arranged_tree = UiArrangedTree {
        tree_id: tree.tree_id.clone(),
        roots: tree.roots.clone(),
        nodes,
        draw_order,
        canvas_layers,
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
) -> bool {
    if checked_node_ids.is_empty()
        || !changed_node_ids.is_subset(checked_node_ids)
        || arranged_tree.tree_id != tree.tree_id
        || arranged_tree.nodes.len() != tree.nodes.len()
        || arranged_tree.roots != tree.roots
        || node_indices.len() != arranged_tree.nodes.len()
        || slot_indices.len() != tree.slots.len()
    {
        return false;
    }

    let mut replacements = Vec::with_capacity(changed_node_ids.len());
    for node_id in checked_node_ids {
        let Some(node) = tree.node(*node_id) else {
            return false;
        };
        if changed_node_ids.contains(node_id) && has_clip_ancestor(tree, *node_id) {
            return false;
        }
        let Some(previous_index) = node_indices.get(node_id).copied() else {
            return false;
        };
        let Some(previous) = arranged_tree.nodes.get(previous_index) else {
            return false;
        };
        let next_frame = node.layout_cache.frame;
        let next_clip_frame =
            effective_node_clip_frame(tree, node.node_id).unwrap_or(node.layout_cache.frame);
        let next_slot = arranged_node_slot_indexed(tree, node.node_id, slot_indices);
        if !same_tree_non_geometry_fields(tree, previous, node, slot_indices) {
            return false;
        }
        if changed_node_ids.contains(node_id) {
            if !same_slot_non_geometry_fields(previous.slot.as_ref(), next_slot.as_ref()) {
                return false;
            }
            replacements.push((previous_index, next_frame, next_clip_frame, next_slot));
        } else if previous.frame != next_frame
            || previous.clip_frame != next_clip_frame
            || previous.slot != next_slot
        {
            return false;
        }
    }

    for (index, frame, clip_frame, slot) in replacements {
        let Some(current) = arranged_tree.nodes.get_mut(index) else {
            return false;
        };
        current.frame = frame;
        current.clip_frame = clip_frame;
        current.slot = slot;
    }
    true
}

pub(crate) fn arranged_node_indices(arranged_tree: &UiArrangedTree) -> BTreeMap<UiNodeId, usize> {
    arranged_tree
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect()
}

pub(crate) fn arranged_slot_indices(tree: &UiTree) -> BTreeMap<UiNodeId, usize> {
    tree.slots
        .iter()
        .enumerate()
        .map(|(index, slot)| (slot.child_id, index))
        .collect()
}

fn arranged_node_from_tree(tree: &UiTree, node_id: UiNodeId) -> UiArrangedNode {
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
        z_index: arranged_node_z_index(tree, node.node_id),
        paint_order: node.paint_order,
        visibility: node.effective_visibility(),
        input_policy: node.input_policy,
        enabled: node.state_flags.enabled,
        clickable: node.state_flags.clickable,
        hoverable: node.state_flags.hoverable,
        focusable: node.is_focus_candidate(),
        clip_to_bounds: node.clip_to_bounds || node.container.clips_to_bounds(),
        control_id: node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.clone()),
        slot: arranged_node_slot(tree, node.node_id),
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
    let slot = tree.slots.get(*slot_indices.get(&child_id)?)?;
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
        .and_then(|index| tree.slots.get(*index))
        .filter(|slot| {
            slot.child_id == child_id
                && matches!(slot.kind, UiSlotKind::Overlay | UiSlotKind::Canvas)
        })
        .map(|slot| slot.z_order)
        .unwrap_or_default();
    node.z_index.saturating_add(slot_z)
}

fn has_clip_ancestor(tree: &UiTree, node_id: UiNodeId) -> bool {
    let Some(node) = tree.node(node_id) else {
        return true;
    };
    if node.clip_to_bounds || node.container.clips_to_bounds() {
        return true;
    }
    let mut current = node.parent;
    while let Some(parent_id) = current {
        let Some(parent) = tree.node(parent_id) else {
            return true;
        };
        if parent.clip_to_bounds || parent.container.clips_to_bounds() {
            return true;
        }
        current = parent.parent;
    }
    false
}

fn arranged_canvas_layers(tree: &UiTree) -> Vec<UiCanvasLayerGroup> {
    let mut layers = Vec::new();
    let mut canvas_parents: Vec<_> = tree
        .nodes
        .values()
        .filter(|node| node.container.child_slot_kind() == Some(UiSlotKind::Canvas))
        .collect();
    canvas_parents.sort_by_key(|node| node.node_id);

    for parent in canvas_parents {
        let mut children: Vec<_> = tree
            .slots
            .iter()
            .filter(|slot| slot.parent_id == parent.node_id && slot.kind == UiSlotKind::Canvas)
            .filter(|slot| parent.children.contains(&slot.child_id))
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

fn arranged_node_slot(
    tree: &UiTree,
    child_id: UiNodeId,
) -> Option<zircon_runtime_interface::ui::surface::UiArrangedSlotSummary> {
    let parent_id = tree.node(child_id)?.parent?;
    let parent = tree.node(parent_id)?;
    let slot_kind = parent.container.child_slot_kind()?;
    tree.slots
        .iter()
        .find(|slot| {
            slot.parent_id == parent_id && slot.child_id == child_id && slot.kind == slot_kind
        })
        .map(Into::into)
}

fn arranged_node_z_index(tree: &UiTree, node_id: UiNodeId) -> i32 {
    let Some(node) = tree.node(node_id) else {
        return 0;
    };
    node.z_index
        .saturating_add(layering_slot_z_order(tree, node_id).unwrap_or_default())
}

fn layering_slot_z_order(tree: &UiTree, child_id: UiNodeId) -> Option<i32> {
    let parent_id = tree.node(child_id)?.parent?;
    tree.slots
        .iter()
        .find(|slot| {
            slot.parent_id == parent_id
                && slot.child_id == child_id
                && matches!(slot.kind, UiSlotKind::Overlay | UiSlotKind::Canvas)
        })
        .map(|slot| slot.z_order)
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
    if !node.is_self_hit_test_visible() {
        return Ok(false);
    }

    let mut current = node.parent;
    while let Some(id) = current {
        let ancestor = arranged_tree.get(id).ok_or(UiTreeError::MissingNode(id))?;
        if !ancestor.allows_child_hit_test() {
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
