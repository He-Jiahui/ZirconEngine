use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    template_component_family, TemplateComponentFamily,
};

pub(super) fn build_template_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    let has_dispatchable = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .any(|node| is_dispatchable(&node));
    has_dispatchable.then(|| template_nodes_surface_frame(nodes, surface_size))
}

pub(super) fn template_nodes_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> UiSurfaceFrame {
    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.template_nodes.hit"));
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        surface_size.width.max(1.0),
        surface_size.height.max(1.0),
    );
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("template_nodes/root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable(&node) {
            continue;
        }
        let component = if node.component_role.is_empty() {
            template_component_family(&node)
                .map(TemplateComponentFamily::as_str)
                .unwrap_or_default()
                .to_string()
        } else {
            node.component_role.to_string()
        };
        let metadata = UiTemplateNodeMetadata {
            component,
            control_id: Some(node.control_id.to_string()),
            ..Default::default()
        };
        let mut tree_node = UiTreeNode::new(
            UiNodeId::new(row as u64 + 2),
            UiNodePath::new(format!("template_nodes/{}", node.node_id)),
        )
        .with_frame(UiFrame::new(
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
        ))
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: !node.disabled,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: node.pressed,
            checked: node.checked,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(metadata);
        tree_node.layout_cache.clip_frame = template_node_clip_frame(&node);
        let _ = surface.tree.insert_child(UiNodeId::new(1), tree_node);
    }

    surface.rebuild();
    surface.surface_frame()
}

fn is_dispatchable(node: &TemplatePaneNodeData) -> bool {
    let family = template_component_family(node);
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || family == Some(TemplateComponentFamily::TextInput))
}

fn template_node_clip_frame(node: &TemplatePaneNodeData) -> Option<UiFrame> {
    node.has_clip_frame.then(|| {
        UiFrame::new(
            node.clip_frame.x,
            node.clip_frame.y,
            node.clip_frame.width,
            node.clip_frame.height,
        )
    })
}
