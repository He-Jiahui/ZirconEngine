use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style::is_button_disabled;

const MATERIAL_STATE_LAYER_OPACITY_HOVER: f32 = 0.08;
const MATERIAL_STATE_LAYER_OPACITY_FOCUS: f32 = 0.10;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MATERIAL_STATE_LAYER_OPACITY_PRESS: f32 = 0.10;
const MATERIAL_STATE_LAYER_OPACITY_DRAG: f32 = 0.16;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_opacity(
    node: &TemplatePaneNodeData,
) -> Option<f32> {
    if !node.state_layer_enabled {
        return None;
    }
    if is_button_disabled(node) {
        return Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS);
    }
    if node.focused || node.selected || node.checked {
        return Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS);
    }
    if node.pressed || node.enter_pressed {
        return Some(MATERIAL_STATE_LAYER_OPACITY_PRESS);
    }
    if node.dragging {
        return Some(MATERIAL_STATE_LAYER_OPACITY_DRAG);
    }
    if node.hovered || node.drop_hovered || node.active_drag_target {
        return Some(MATERIAL_STATE_LAYER_OPACITY_HOVER);
    }
    None
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.state_layer_color.a > 0 {
        [
            node.state_layer_color.r,
            node.state_layer_color.g,
            node.state_layer_color.b,
            node.state_layer_color.a,
        ]
    } else {
        PALETTE.focus_ring
    }
}
