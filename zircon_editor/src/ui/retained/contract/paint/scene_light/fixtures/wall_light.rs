use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::{surface_color, template_corner_radius};
use super::super::primitives::{color_with_alpha_factor, push_expanded_layer};
use super::palette::{WALL_LIGHT_CORE, WALL_LIGHT_HALO};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_wall_light(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        WALL_LIGHT_HALO,
        8.0,
        10.0,
        14.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color_with_alpha_factor(color, 0.74)),
        None,
        0.0,
        template_corner_radius(node),
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 1.0,
            width: (rect.width - 6.0).max(1.0),
            height: (rect.height * 0.36).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(WALL_LIGHT_CORE),
        None,
        0.0,
        3.0,
        opacity,
    ));
}
