use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::tooltip_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_bubble_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = tooltip_metrics();
    FrameRect {
        x: rect.x + (rect.width - metrics.bubble_width).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: metrics.bubble_width.min(rect.width.max(1.0)),
        height: metrics.bubble_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round(),
        height: rect.height.round(),
    }
}
