use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::material_state_layer::push_state_layer_commands;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::{
    border_color, draws_elevation_shadow, elevation_shadow_rect, surface_color,
    template_border_width, template_corner_radius,
};
use super::eligibility::draws_border;

const MATERIAL_ELEVATION_SHADOW_OPACITY: f32 = 0.72;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
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
