use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
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
    if node.pressed || node.enter_pressed {
        return Some(MATERIAL_STATE_LAYER_OPACITY_PRESS);
    }
    if node.dragging {
        return Some(MATERIAL_STATE_LAYER_OPACITY_DRAG);
    }
    if node.focused || node.selected || node.checked {
        return Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS);
    }
    if node.hovered || node.drop_hovered || node.active_drag_target {
        return Some(MATERIAL_STATE_LAYER_OPACITY_HOVER);
    }
    None
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    state_layer_color_from_host(node, current_host_palette())
}

fn state_layer_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.state_layer_color.a > 0 {
        [
            node.state_layer_color.r,
            node.state_layer_color.g,
            node.state_layer_color.b,
            node.state_layer_color.a,
        ]
    } else {
        palette.focus_ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use crate::ui::retained_host::primitives::Color;

    #[test]
    fn state_layer_fallback_color_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.focus_ring = [10, 11, 12, 255];
        let node = TemplatePaneNodeData::default();

        assert_eq!(
            state_layer_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
    }

    #[test]
    fn state_layer_declared_color_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.state_layer_color = Color::from_argb_u8(128, 20, 21, 22);

        assert_eq!(
            state_layer_color_from_host(&node, palette),
            [20, 21, 22, 128]
        );
    }

    #[test]
    fn pressed_state_layer_opacity_has_priority_over_focus_and_selection() {
        let node = TemplatePaneNodeData {
            state_layer_enabled: true,
            focused: true,
            selected: true,
            checked: true,
            pressed: true,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            state_layer_opacity(&node),
            Some(MATERIAL_STATE_LAYER_OPACITY_PRESS)
        );
    }

    #[test]
    fn drag_state_layer_opacity_has_priority_over_focus_and_selection() {
        let node = TemplatePaneNodeData {
            state_layer_enabled: true,
            focused: true,
            selected: true,
            checked: true,
            dragging: true,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            state_layer_opacity(&node),
            Some(MATERIAL_STATE_LAYER_OPACITY_DRAG)
        );
    }
}
