use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::component_variant_contains;

const SKELETON_TEXT_SCALE_Y: f32 = 0.60;
const SKELETON_DEFAULT_RADIUS: f32 = 4.0;
const SKELETON_WAVE_X_RATIO: f32 = 0.28;
const SKELETON_WAVE_WIDTH_RATIO: f32 = 0.22;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_frame_for_variant(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    if component_variant_contains(node, "circular") {
        let size = rect.width.min(rect.height).max(1.0);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    if component_variant_contains(node, "text") {
        let height = (rect.height * SKELETON_TEXT_SCALE_Y).max(1.0);
        return FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - height) * 0.5,
            width: rect.width,
            height,
        };
    }
    rect.clone()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    if component_variant_contains(node, "rectangular") {
        return 0.0;
    }
    if component_variant_contains(node, "circular") {
        return rect.width.min(rect.height) * 0.5;
    }
    let configured = configured_corner_radius(node).unwrap_or(SKELETON_DEFAULT_RADIUS);
    configured.min(rect.height * 0.5).max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_wave_frame(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width * SKELETON_WAVE_X_RATIO,
        y: rect.y,
        width: (rect.width * SKELETON_WAVE_WIDTH_RATIO).max(1.0),
        height: rect.height,
    }
}

fn configured_corner_radius(node: &TemplatePaneNodeData) -> Option<f32> {
    let radius = node
        .button_style
        .element
        .corner_radius
        .max(node.corner_radius);
    (radius.is_finite() && radius > 0.0).then_some(radius)
}
