use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{avatar_border_color, avatar_border_width};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_background(
    commands: &mut Vec<HostPaintCommand>,
    avatar_rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    background: [u8; 4],
    corner_radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        avatar_rect,
        Some(clip.clone()),
        order,
        Some(background),
        None,
        0.0,
        corner_radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_border(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    avatar_rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    corner_radius: f32,
    opacity: f32,
) {
    let Some(border_color) = avatar_border_color(node) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        avatar_rect,
        Some(clip.clone()),
        order,
        None,
        Some(border_color),
        avatar_border_width(node),
        corner_radius,
        opacity,
    ));
}
