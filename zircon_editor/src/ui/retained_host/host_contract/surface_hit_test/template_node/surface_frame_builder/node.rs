use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::super::super::super::data::TemplatePaneNodeData;
use super::dispatch::template_component;

pub(super) fn template_surface_tree_node(
    row: usize,
    node: &TemplatePaneNodeData,
    dispatchable: bool,
) -> UiTreeNode {
    let metadata = UiTemplateNodeMetadata {
        component: template_component(node),
        control_id: Some(node.control_id.to_string()),
        ..Default::default()
    };
    let mut tree_node = UiTreeNode::new(
        UiNodeId::new(row as u64 + 2),
        template_node_path(node.node_id.as_str()),
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
        clickable: dispatchable,
        hoverable: dispatchable,
        focusable: dispatchable,
        pressed: node.pressed,
        checked: node.checked,
        dirty: false,
    })
    .with_input_policy(if dispatchable {
        UiInputPolicy::Receive
    } else {
        UiInputPolicy::Ignore
    })
    .with_template_metadata(metadata);
    tree_node.layout_cache.clip_frame = template_node_clip_frame(node);
    tree_node
}

fn template_node_path(node_id: &str) -> UiNodePath {
    const PREFIX: &str = "template_nodes/";
    let mut path = String::with_capacity(PREFIX.len() + node_id.len());
    path.push_str(PREFIX);
    path.push_str(node_id);
    UiNodePath::new(path)
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

#[cfg(test)]
#[path = "tests/node_optimization_batch_fi_tests.rs"]
mod optimization_batch_fi_tests;
