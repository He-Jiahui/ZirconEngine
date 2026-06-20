use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

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
    FrameRect {
        x: rect.x + chip_leading_margin(node),
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}
