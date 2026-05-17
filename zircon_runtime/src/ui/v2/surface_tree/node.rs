use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode};
use zircon_runtime_interface::ui::v2::{
    UiV2ArenaNode, UiV2AssetError, UiV2NodeArena, UiV2NodeHandle, UiV2ResolvedStyleSheet,
};

use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::interaction::infer_interaction;
use super::layout::{infer_container, infer_layout_contract};
use super::slot::infer_slot_contract;

pub(in crate::ui::v2) fn build_tree_from_arena(
    asset_id: &str,
    tree_id: UiTreeId,
    arena: &UiV2NodeArena,
    resolved_styles: &UiV2ResolvedStyleSheet,
) -> Result<UiTree, UiV2AssetError> {
    let mut tree = UiTree::new(tree_id);
    let Some(root) = arena.root else {
        return Ok(tree);
    };

    let root_node = arena
        .node(root)
        .ok_or_else(|| missing_handle(asset_id, root))?;
    let mut next_node_id = 1;
    let mut stack = vec![BuildFrame {
        handle: root,
        parent_id: None,
        path: stable_node_path(root_node),
        slot: BTreeMap::new(),
    }];
    while let Some(frame) = stack.pop() {
        let arena_node = arena
            .node(frame.handle)
            .ok_or_else(|| missing_handle(asset_id, frame.handle))?;
        let node_id = insert_arena_node(
            asset_id,
            &mut tree,
            frame.parent_id,
            arena_node,
            resolved_styles,
            frame.slot.clone(),
            &frame.path,
            &mut next_node_id,
        )?;

        for (index, child) in arena_node.children.iter().enumerate().rev() {
            let child_node = arena
                .node(child.child)
                .ok_or_else(|| missing_handle(asset_id, child.child))?;
            stack.push(BuildFrame {
                handle: child.child,
                parent_id: Some(node_id),
                path: stable_child_path(child_node, index),
                slot: child.slot.clone(),
            });
        }
    }

    if tree.roots.is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: "compiled v2 arena did not insert a root node".to_string(),
        });
    }
    Ok(tree)
}

fn insert_arena_node(
    asset_id: &str,
    tree: &mut UiTree,
    parent_id: Option<UiNodeId>,
    node: &UiV2ArenaNode,
    resolved_styles: &UiV2ResolvedStyleSheet,
    slot_attributes: BTreeMap<String, Value>,
    path: &str,
    next_node_id: &mut u64,
) -> Result<UiNodeId, UiV2AssetError> {
    let node_id = UiNodeId::new(*next_node_id);
    *next_node_id += 1;

    let parent_container = parent_id
        .and_then(|parent_id| tree.node(parent_id))
        .map(|parent| parent.container);
    let attributes = arena_node_attributes(node, resolved_styles);
    let style_overrides = arena_node_style_overrides(node, resolved_styles);
    let layout = infer_layout_contract(
        asset_id,
        path,
        attributes.get("layout"),
        &slot_attributes,
        parent_container,
    )?;
    let container = layout
        .container
        .unwrap_or_else(|| infer_container(&node.component));
    let (state_flags, input_policy) = infer_interaction(node, &attributes);
    let mut tree_node = UiTreeNode::new(node_id, UiNodePath::new(path.to_string()))
        .with_state_flags(state_flags)
        .with_constraints(layout.constraints)
        .with_anchor(layout.anchor)
        .with_pivot(layout.pivot)
        .with_position(layout.position)
        .with_input_policy(layout.input_policy.unwrap_or(input_policy))
        .with_layout_boundary(layout.layout_boundary)
        .with_layout_stretch_axes(layout.stretch_width, layout.stretch_height)
        .with_z_index(layout.z_index)
        .with_container(container)
        .with_clip_to_bounds(layout.clip_to_bounds || container.clips_to_bounds())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: node.component.clone(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            attributes,
            slot_attributes: slot_attributes.clone(),
            style_overrides,
            style_tokens: BTreeMap::new(),
            bindings: node.events.clone(),
            a11y: Default::default(),
            widget: Default::default(),
        });
    if container.is_scrollable() {
        tree_node = tree_node.with_scroll_state(Default::default());
    }

    if let Some(parent_id) = parent_id {
        let parent_container = tree
            .node(parent_id)
            .map(|parent| parent.container)
            .ok_or_else(|| UiV2AssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!("ui tree is missing parent {parent_id:?}"),
            })?;
        let slot = infer_slot_contract(
            asset_id,
            path,
            parent_id,
            node_id,
            parent_container,
            &slot_attributes,
        )?;
        tree.insert_child(parent_id, tree_node).map_err(|error| {
            UiV2AssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: error.to_string(),
            }
        })?;
        tree.slots.push(slot);
    } else {
        tree.insert_root(tree_node);
    }

    Ok(node_id)
}

fn arena_node_attributes(
    node: &UiV2ArenaNode,
    resolved_styles: &UiV2ResolvedStyleSheet,
) -> BTreeMap<String, Value> {
    let mut attributes = node.props.clone();
    attributes.extend(node.state.clone());
    if let Some(layout) = &node.layout {
        attributes.insert(
            "layout".to_string(),
            Value::Table(layout.clone().into_iter().collect()),
        );
    }
    if let Some(resolved) = resolved_styles.nodes.get(&node.source_id) {
        attributes.extend(resolved.self_values.clone());
    }
    attributes
}

fn arena_node_style_overrides(
    node: &UiV2ArenaNode,
    resolved_styles: &UiV2ResolvedStyleSheet,
) -> BTreeMap<String, Value> {
    let mut style_overrides = BTreeMap::new();
    if let Some(resolved) = resolved_styles.nodes.get(&node.source_id) {
        style_overrides.extend(resolved.self_values.clone());
    }
    style_overrides.extend(node.style.self_values.clone());
    style_overrides
}

fn stable_node_path(node: &UiV2ArenaNode) -> String {
    format!("v2/{}", node.source_id)
}

fn stable_child_path(node: &UiV2ArenaNode, index: usize) -> String {
    node.control_id
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|control_id| format!("v2/{control_id}"))
        .unwrap_or_else(|| format!("v2/{}[{index}]", node.source_id))
}

fn missing_handle(asset_id: &str, handle: UiV2NodeHandle) -> UiV2AssetError {
    UiV2AssetError::MissingNode {
        asset_id: asset_id.to_string(),
        node_id: format!("handle {}", handle.index()),
    }
}

struct BuildFrame {
    handle: UiV2NodeHandle,
    parent_id: Option<UiNodeId>,
    path: String,
    slot: BTreeMap<String, Value>,
}
