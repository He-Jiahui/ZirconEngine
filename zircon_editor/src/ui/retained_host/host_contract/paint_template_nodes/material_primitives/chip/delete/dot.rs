use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_chip_delete_dot(
    commands: &mut Vec<HostPaintCommand>,
    center_x: f32,
    center_y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    stroke: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - stroke * 0.5,
            y: center_y - stroke * 0.5,
            width: stroke,
            height: stroke,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        stroke * 0.5,
        opacity,
    ));
}
