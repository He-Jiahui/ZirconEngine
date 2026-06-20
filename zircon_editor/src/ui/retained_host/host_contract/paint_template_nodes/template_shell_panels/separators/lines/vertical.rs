use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_left_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_right_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x + rect.width - 1.0,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    height: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x.round(),
            y: y.round(),
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
