use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::{
    frame_from_template, intersect, is_visible_frame, translated,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_rect_and_clip(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    pane_clip: &FrameRect,
) -> Option<(FrameRect, FrameRect)> {
    let local = frame_from_template(&node.frame);
    let rect = translated(&local, origin.x, origin.y);
    if !is_visible_frame(&rect) {
        return None;
    }
    let node_clip = template_node_clip(node, origin, pane_clip)?;
    intersect(&rect, &node_clip).map(|_| (rect, node_clip))
}

fn template_node_clip(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    pane_clip: &FrameRect,
) -> Option<FrameRect> {
    let node_clip = if node.has_clip_frame {
        translated(&frame_from_template(&node.clip_frame), origin.x, origin.y)
    } else {
        pane_clip.clone()
    };
    intersect(&node_clip, pane_clip)
}
