use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::material_state_layer::push_state_layer_commands;
use super::render_commands::HostPaintCommand;
use super::template_style::{
    border_color, draws_elevation_shadow, elevation_shadow_rect, is_mui_overlay_surface_node,
    surface_color, template_border_width, template_corner_radius,
};

const MATERIAL_ELEVATION_SHADOW_OPACITY: f32 = 0.72;

pub(super) fn push_template_surface_fallback_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    surface_already_drawn: bool,
) {
    if surface_already_drawn || !draws_surface(node) {
        return;
    }

    let border_width = template_border_width(node);
    let corner_radius = template_corner_radius(node);
    if draws_elevation_shadow(node) {
        commands.push(HostPaintCommand::quad(
            elevation_shadow_rect(rect, node.elevation),
            Some(clip.clone()),
            order - 1,
            Some(PALETTE.shadow),
            None,
            0.0,
            corner_radius,
            MATERIAL_ELEVATION_SHADOW_OPACITY * opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(surface_color(node)),
        draws_border(node).then_some(border_color(node)),
        border_width,
        corner_radius,
        opacity,
    ));
    push_state_layer_commands(
        commands,
        node,
        rect,
        clip,
        corner_radius,
        order + 1,
        opacity,
    );
}

fn draws_surface(node: &TemplatePaneNodeData) -> bool {
    if is_frame_only_node(node) {
        return false;
    }
    matches!(node.role.as_str(), "Panel" | "Button" | "Mount")
        || is_mui_overlay_surface_node(node)
        || !node.surface_variant.is_empty()
        || !node.button_variant.is_empty()
        || node.button_style.element.background_color.is_some()
        || node.button_style.element.border_color.is_some()
        || node.button_style.element.border_width > 0.0
        || node.button_style.element.corner_radius > 0.0
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.hovered
        || node.pressed
        || node.focused
        || node.state_layer_enabled
        || node.ripple_enabled
        || node.disabled
}

pub(super) fn is_frame_only_node(node: &TemplatePaneNodeData) -> bool {
    node.surface_variant
        .split_whitespace()
        .any(|part| matches!(part, "frame_only" | "frame-only" | "frameOnly"))
}

fn draws_border(node: &TemplatePaneNodeData) -> bool {
    node.button_style.element.border_width > 0.0
        || node.button_style.element.border_color.is_some()
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.checked
        || node.focused
        || node.hovered
        || node.pressed
        || node.drop_hovered
        || node.active_drag_target
        || matches!(node.role.as_str(), "Button" | "Mount")
}
