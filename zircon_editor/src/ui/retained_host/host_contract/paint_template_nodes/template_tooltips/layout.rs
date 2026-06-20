use super::super::super::data::{FrameRect, TemplatePaneNodeData};

const TOOLTIP_BUBBLE_WIDTH: f32 = 96.0;
const TOOLTIP_BUBBLE_HEIGHT: f32 = 45.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_bubble_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + (rect.width - TOOLTIP_BUBBLE_WIDTH).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: TOOLTIP_BUBBLE_WIDTH.min(rect.width.max(1.0)),
        height: TOOLTIP_BUBBLE_HEIGHT,
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
