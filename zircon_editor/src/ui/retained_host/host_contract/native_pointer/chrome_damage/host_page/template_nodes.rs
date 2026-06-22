use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplatePaneNodeData,
};

use super::super::union::union_visible_frame;

pub(super) fn union_host_page_template_node_damage(
    mut damage: Option<FrameRect>,
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let template_nodes = &presentation.host_scene_data.page_chrome.template_nodes;
    for row in 0..template_nodes.row_count() {
        let Some(node) = template_nodes.row_data(row) else {
            continue;
        };
        damage = union_visible_frame(damage, template_node_frame(&node));
    }
    damage
}

fn template_node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}
