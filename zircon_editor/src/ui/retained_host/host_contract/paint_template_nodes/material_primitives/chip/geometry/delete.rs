use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::chip_is_small;
use super::metrics::{chip_bounded_extent, CHIP_DELETE_MEDIUM_EDGE, CHIP_DELETE_SMALL_EDGE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_delete_icon_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let edge = chip_delete_icon_edge(node, rect);
    let width = chip_bounded_extent(rect.width);
    let height = chip_bounded_extent(rect.height);
    let preferred_right_margin: f32 = if chip_is_small(node) { 4.0 } else { 5.0 };
    let right_margin = preferred_right_margin.min((width - edge).max(0.0));
    FrameRect {
        x: rect.x + width - right_margin - edge,
        y: rect.y + (height - edge) * 0.5,
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
    let width = chip_bounded_extent(rect.width);
    let content_height = (chip_bounded_extent(rect.height) - 4.0).max(0.0);
    chip_delete_edge(node).min(width).min(content_height)
}

#[cfg(test)]
#[path = "tests/delete.rs"]
mod tests;
