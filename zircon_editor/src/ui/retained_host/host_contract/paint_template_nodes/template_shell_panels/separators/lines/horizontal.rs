use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_geometry::intersect;
use super::super::super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_top_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(commands, rect, rect.y, clip, order, color, opacity);
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_bottom_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(
        commands,
        rect,
        rect.y + rect.height - 1.0,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let line = FrameRect {
        x: rect.x,
        y: y.round(),
        width: rect.width,
        height: 1.0,
    };
    if intersect(&line, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        line,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
