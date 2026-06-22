use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const RIPPLE_DIAMETER_EXPANSION: f32 =
    2.0 * std::f32::consts::SQRT_2;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn ripple_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let diameter = ripple_diameter(rect);
    let center_x = if node.ripple_pressed_x.is_finite() {
        rect.x + node.ripple_pressed_x
    } else {
        rect.x + rect.width * 0.5
    };
    let center_y = if node.ripple_pressed_y.is_finite() {
        rect.y + node.ripple_pressed_y
    } else {
        rect.y + rect.height * 0.5
    };
    FrameRect {
        x: center_x - diameter * 0.5,
        y: center_y - diameter * 0.5,
        width: diameter,
        height: diameter,
    }
}

pub(super) fn ripple_clip(
    node: &TemplatePaneNodeData,
    clip: &FrameRect,
    rect: &FrameRect,
) -> Option<FrameRect> {
    if !node.ripple_unclipped {
        intersect(clip, rect)
    } else {
        Some(clip.clone())
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn ripple_diameter(
    rect: &FrameRect,
) -> f32 {
    rect.width * RIPPLE_DIAMETER_EXPANSION
}

pub(super) fn ripple_radius(rect: &FrameRect) -> f32 {
    ripple_diameter(rect) * 0.5
}
