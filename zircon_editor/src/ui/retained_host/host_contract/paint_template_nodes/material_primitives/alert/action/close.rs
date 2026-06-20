use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::metrics::ALERT_CLOSE_DOT_EDGE;

pub(super) fn push_alert_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    frame: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let start_x = frame.x + frame.width * 0.28;
    let end_x = frame.x + frame.width * 0.72;
    let start_y = frame.y + frame.height * 0.28;
    let end_y = frame.y + frame.height * 0.72;
    for index in 0..5 {
        let ratio = index as f32 / 4.0;
        push_alert_close_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            start_y + (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
        );
        push_alert_close_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            end_y - (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
        );
    }
}

fn push_alert_close_dot(
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
            x: center_x - ALERT_CLOSE_DOT_EDGE * 0.5,
            y: center_y - ALERT_CLOSE_DOT_EDGE * 0.5,
            width: ALERT_CLOSE_DOT_EDGE,
            height: ALERT_CLOSE_DOT_EDGE,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        ALERT_CLOSE_DOT_EDGE * 0.5,
        opacity,
    ));
}
