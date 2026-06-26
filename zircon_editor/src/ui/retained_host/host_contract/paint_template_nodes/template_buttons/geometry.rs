use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::identity::is_add_component_button;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

const ADD_COMPONENT_OFFSET_Y: f32 = 1.5;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    if is_add_component_button(node) {
        rect.y += ADD_COMPONENT_OFFSET_Y;
    }
    rect
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let radius = if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        node.corner_radius
    } else {
        METRICS.radius_control
    };
    radius.min(rect.height * 0.5).max(0.0)
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
