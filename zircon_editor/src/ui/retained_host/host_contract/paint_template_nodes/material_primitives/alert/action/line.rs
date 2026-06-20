use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_alert_action_line(
    commands: &mut Vec<HostPaintCommand>,
    frame: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: frame.x + 3.0,
            y: frame.y + frame.height * 0.5 - 1.0,
            width: frame.width - 6.0,
            height: 2.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
