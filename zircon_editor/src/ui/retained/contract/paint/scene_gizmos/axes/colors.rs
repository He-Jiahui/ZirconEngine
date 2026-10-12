use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_style::text_color;
use super::super::super::template_style_color::resolved_style_color;

const AXIS_X: [u8; 4] = [239, 73, 63, 255];
const AXIS_Y: [u8; 4] = [88, 208, 94, 255];
const AXIS_Z: [u8; 4] = [57, 144, 255, 255];
pub(super) const AXIS_GLOW: [u8; 4] = [34, 193, 203, 64];

pub(super) fn axis_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if let Some(color) = declared_surface_color(node) {
        return color;
    }
    match node.control_id.as_str() {
        id if id.contains("AxisX") => AXIS_X,
        id if id.contains("AxisY") => AXIS_Y,
        id if id.contains("AxisZ") => AXIS_Z,
        _ => text_color(node),
    }
}

pub(super) fn axis_glow(color: [u8; 4]) -> [u8; 4] {
    [color[0], color[1], color[2], 58]
}

fn declared_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .filter(|color| color[3] > 0)
}
