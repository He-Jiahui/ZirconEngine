use super::super::super::super::data::TemplatePaneNodeData;
use super::super::resolved_style_color;

const SKELETON_DEFAULT_BG: [u8; 4] = [58, 66, 73, 255];
const SKELETON_WAVE: [u8; 4] = [255, 255, 255, 36];
const SKELETON_DISABLED_OPACITY: f32 = 0.56;

pub(super) fn skeleton_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(SKELETON_DEFAULT_BG)
}

pub(super) fn skeleton_wave_color() -> [u8; 4] {
    SKELETON_WAVE
}

pub(super) fn skeleton_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (skeleton_border_width(node) > 0.0).then_some(SKELETON_DEFAULT_BG))
}

pub(super) fn skeleton_border_width(node: &TemplatePaneNodeData) -> f32 {
    let width = node
        .button_style
        .element
        .border_width
        .max(node.border_width);
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn skeleton_opacity(node: &TemplatePaneNodeData) -> f32 {
    if node.disabled {
        SKELETON_DISABLED_OPACITY
    } else {
        1.0
    }
}
