use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::geometry::intersect;
use super::render_commands::HostPaintCommand;
use super::template_nodes::is_button_disabled;
use super::theme::PALETTE;

const MATERIAL_STATE_LAYER_OPACITY_HOVER: f32 = 0.08;
const MATERIAL_STATE_LAYER_OPACITY_FOCUS: f32 = 0.10;
const MATERIAL_STATE_LAYER_OPACITY_PRESS: f32 = 0.10;
const MATERIAL_STATE_LAYER_OPACITY_DRAG: f32 = 0.16;
const RIPPLE_DIAMETER_EXPANSION: f32 = 2.0 * std::f32::consts::SQRT_2;

pub(super) fn push_state_layer_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    corner_radius: f32,
    order: i32,
) {
    let color = state_layer_color(node);
    if let Some(opacity) = state_layer_opacity(node) {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            corner_radius,
            opacity,
        ));
    }

    if node.ripple_enabled && !is_button_disabled(node) && (node.pressed || node.enter_pressed) {
        commands.push(HostPaintCommand::quad(
            ripple_rect(node, rect),
            ripple_clip(node, clip, rect),
            order + 1,
            Some(color),
            None,
            0.0,
            ripple_radius(rect),
            MATERIAL_STATE_LAYER_OPACITY_PRESS,
        ));
    }
}

fn state_layer_opacity(node: &TemplatePaneNodeData) -> Option<f32> {
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

fn state_layer_color(node: &TemplatePaneNodeData) -> [u8; 4] {
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

fn ripple_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
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

fn ripple_clip(
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

fn ripple_diameter(rect: &FrameRect) -> f32 {
    rect.width * RIPPLE_DIAMETER_EXPANSION
}

fn ripple_radius(rect: &FrameRect) -> f32 {
    ripple_diameter(rect) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripple_diameter_matches_slint_width_based_source_contract() {
        let rect = frame(4.0, 8.0, 20.0, 80.0);
        let ripple = ripple_rect(&TemplatePaneNodeData::default(), &rect);
        let expected = 20.0 * RIPPLE_DIAMETER_EXPANSION;

        assert_eq!(ripple.width, expected);
        assert_eq!(ripple.height, expected);
    }

    #[test]
    fn ripple_origin_preserves_zero_press_coordinates() {
        let node = TemplatePaneNodeData {
            ripple_pressed_x: 0.0,
            ripple_pressed_y: 0.0,
            ..TemplatePaneNodeData::default()
        };
        let rect = frame(10.0, 20.0, 24.0, 24.0);
        let ripple = ripple_rect(&node, &rect);
        let radius = ripple_diameter(&rect) * 0.5;

        assert_eq!(ripple.x + radius, rect.x);
        assert_eq!(ripple.y + radius, rect.y);
    }

    #[test]
    fn ripple_enabled_does_not_imply_full_state_layer_overlay() {
        let node = TemplatePaneNodeData {
            ripple_enabled: true,
            pressed: true,
            ..TemplatePaneNodeData::default()
        };
        let rect = frame(0.0, 0.0, 32.0, 20.0);
        let clip = rect.clone();
        let mut commands = Vec::new();

        push_state_layer_commands(&mut commands, &node, &rect, &clip, 4.0, 0);

        assert_eq!(commands.len(), 1);
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
