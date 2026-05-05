use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiArrangedNode, UiArrangedTree},
    tree::{UiInputPolicy, UiTree, UiTreeError},
};

pub fn build_arranged_tree(tree: &UiTree) -> UiArrangedTree {
    let mut nodes: Vec<_> = tree
        .nodes
        .values()
        .map(|node| {
            let visibility = node.effective_visibility();
            UiArrangedNode {
                node_id: node.node_id,
                node_path: node.node_path.clone(),
                parent: node.parent,
                children: node.children.clone(),
                frame: node.layout_cache.frame,
                clip_frame: effective_node_clip_frame(tree, node.node_id)
                    .unwrap_or(node.layout_cache.frame),
                z_index: node.z_index,
                paint_order: node.paint_order,
                visibility,
                input_policy: node.input_policy,
                enabled: node.state_flags.enabled,
                clickable: node.state_flags.clickable,
                hoverable: node.state_flags.hoverable,
                focusable: node.state_flags.focusable,
                clip_to_bounds: node.clip_to_bounds || node.container.clips_to_bounds(),
                control_id: node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.clone()),
            }
        })
        .collect();
    nodes.sort_by_key(|node| (node.z_index, node.paint_order, node.node_id));
    let draw_order = nodes.iter().map(|node| node.node_id).collect();
    UiArrangedTree {
        tree_id: tree.tree_id.clone(),
        roots: tree.roots.clone(),
        nodes,
        draw_order,
    }
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
