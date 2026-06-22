use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::metrics::CHIP_DELETE_STROKE;

pub(super) fn push_chip_delete_dot(
    commands: &mut Vec<HostPaintCommand>,
    center_x: f32,
    center_y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - CHIP_DELETE_STROKE * 0.5,
            y: center_y - CHIP_DELETE_STROKE * 0.5,
            width: CHIP_DELETE_STROKE,
            height: CHIP_DELETE_STROKE,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        CHIP_DELETE_STROKE * 0.5,
        opacity,
    ));
}
