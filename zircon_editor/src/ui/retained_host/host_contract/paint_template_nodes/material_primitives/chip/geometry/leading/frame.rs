use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::metrics::chip_bounded_extent;
use super::edge::{chip_avatar_edge, chip_icon_edge};
use super::margin::chip_leading_margin;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_avatar_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    chip_leading_slot_frame(node, rect, chip_avatar_edge(node, rect))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_icon_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    chip_leading_slot_frame(node, rect, chip_icon_edge(node, rect))
}

fn chip_leading_slot_frame(node: &TemplatePaneNodeData, rect: &FrameRect, edge: f32) -> FrameRect {
    let width = chip_bounded_extent(rect.width);
    let height = chip_bounded_extent(rect.height);
    let edge = edge.min(width).min(height);
    let max_x = rect.x + (width - edge).max(0.0);
    let requested_x = rect.x + chip_bounded_extent(chip_leading_margin(node));
    FrameRect {
        x: requested_x.min(max_x).max(rect.x),
        y: rect.y + (height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;
