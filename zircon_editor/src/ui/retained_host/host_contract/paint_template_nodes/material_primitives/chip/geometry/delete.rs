use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::chip_is_small;
use super::metrics::{CHIP_DELETE_MEDIUM_EDGE, CHIP_DELETE_SMALL_EDGE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_delete_icon_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let edge = chip_delete_icon_edge(node, rect);
    let right_margin = if chip_is_small(node) { 4.0 } else { 5.0 };
    FrameRect {
        x: rect.x + rect.width - right_margin - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_delete_edge(
    node: &TemplatePaneNodeData,
) -> f32 {
    if chip_is_small(node) {
        CHIP_DELETE_SMALL_EDGE
    } else {
        CHIP_DELETE_MEDIUM_EDGE
    }
}

fn chip_delete_icon_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    chip_delete_edge(node).min(rect.height - 4.0).max(1.0)
}
