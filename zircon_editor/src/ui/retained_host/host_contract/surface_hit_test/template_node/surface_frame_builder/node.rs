use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::super::super::super::data::TemplatePaneNodeData;
use super::dispatch::template_component;

pub(super) fn template_surface_tree_node(row: usize, node: &TemplatePaneNodeData) -> UiTreeNode {
    let metadata = UiTemplateNodeMetadata {
        component: template_component(node),
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
    tree_node.layout_cache.clip_frame = template_node_clip_frame(node);
    tree_node
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
