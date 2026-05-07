use std::collections::{BTreeMap, BTreeSet};

use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    tree::{UiTree, UiTreeError},
};

use super::{
    arrange::arrange_node,
    child_frame::free_child_frame,
    measure::measure_node,
    slot::slot_for_container_child,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiIncrementalLayoutStats {
    pub visited_node_count: usize,
    pub geometry_changed_node_count: usize,
    pub skipped_node_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LayoutGeometry {
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
}

pub fn compute_incremental_layout_tree(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<UiIncrementalLayoutStats, UiTreeError> {
    let previous = snapshot_geometry(tree);
    let roots = incremental_layout_roots(tree)?;
    let mut visited = BTreeSet::new();

    for root_id in roots {
        collect_subtree_nodes(tree, root_id, &mut visited)?;
        measure_node(tree, root_id)?;
        arrange_layout_root(tree, root_id, root_size)?;
    }

    let geometry_changed_node_count = tree
        .nodes
        .iter()
        .filter(|(node_id, node)| {
            previous.get(node_id).copied().unwrap_or_default()
                != LayoutGeometry {
                    frame: node.layout_cache.frame,
                    clip_frame: node.layout_cache.clip_frame,
                }
        })
        .count();

    Ok(UiIncrementalLayoutStats {
        visited_node_count: visited.len(),
        geometry_changed_node_count,
        skipped_node_count: tree.nodes.len().saturating_sub(visited.len()),
    })
}

fn incremental_layout_roots(tree: &UiTree) -> Result<Vec<UiNodeId>, UiTreeError> {
    let candidates = tree
        .nodes
        .values()
        .filter(|node| {
            node.dirty.layout || node.dirty.style || node.dirty.text || node.dirty.visible_range
        })
        .map(|node| propagated_layout_root(tree, node.node_id))
        .collect::<Result<BTreeSet<_>, _>>()?;

    let mut roots = Vec::new();
    for candidate in candidates.iter().copied() {
        if !has_ancestor_in(candidate, &candidates, tree)? {
            roots.push(candidate);
        }
    }
    Ok(roots)
}

fn propagated_layout_root(tree: &UiTree, node_id: UiNodeId) -> Result<UiNodeId, UiTreeError> {
    let mut current = node_id;
    let mut root = node_id;
    while let Some(parent_id) = tree
        .node(current)
        .ok_or(UiTreeError::MissingNode(current))?
        .parent
    {
        let parent = tree
            .node(parent_id)
            .ok_or(UiTreeError::MissingParent(parent_id))?;
        if !(parent.layout_boundary.propagates_child_layout_invalidation()
            || parent.container.is_auto_layout_container())
        {
            break;
        }
        root = parent_id;
        current = parent_id;
    }
    Ok(root)
}

fn has_ancestor_in(
    node_id: UiNodeId,
    roots: &BTreeSet<UiNodeId>,
    tree: &UiTree,
) -> Result<bool, UiTreeError> {
    let mut current = node_id;
    while let Some(parent_id) = tree
        .node(current)
        .ok_or(UiTreeError::MissingNode(current))?
        .parent
    {
        if roots.contains(&parent_id) {
            return Ok(true);
        }
        current = parent_id;
    }
    Ok(false)
}

fn arrange_layout_root(
    tree: &mut UiTree,
    root_id: UiNodeId,
    root_size: UiSize,
) -> Result<(), UiTreeError> {
    let parent_id = tree
        .node(root_id)
        .ok_or(UiTreeError::MissingNode(root_id))?
        .parent;
    let Some(parent_id) = parent_id else {
        return arrange_node(tree, root_id, root_frame(root_size), None);
    };

    let parent = tree
        .node(parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?;
    let parent_frame = parent.layout_cache.frame;
    let inherited_clip = parent.layout_cache.clip_frame;
    let parent_container = parent.container;
    let child_frame = free_child_frame(
        tree,
        root_id,
        parent_frame,
        slot_for_container_child(tree, parent_id, root_id, parent_container),
    )?;

    arrange_node(tree, root_id, child_frame, inherited_clip)
}

fn collect_subtree_nodes(
    tree: &UiTree,
    node_id: UiNodeId,
    visited: &mut BTreeSet<UiNodeId>,
) -> Result<(), UiTreeError> {
    let node = tree.node(node_id).ok_or(UiTreeError::MissingNode(node_id))?;
    visited.insert(node_id);
    for child_id in &node.children {
        collect_subtree_nodes(tree, *child_id, visited)?;
    }
    Ok(())
}

fn snapshot_geometry(tree: &UiTree) -> BTreeMap<UiNodeId, LayoutGeometry> {
    tree.nodes
        .iter()
        .map(|(node_id, node)| {
            (
                *node_id,
                LayoutGeometry {
                    frame: node.layout_cache.frame,
                    clip_frame: node.layout_cache.clip_frame,
                },
            )
        })
        .collect()
}

fn root_frame(root_size: UiSize) -> UiFrame {
    UiFrame::new(0.0, 0.0, root_size.width.max(0.0), root_size.height.max(0.0))
}
