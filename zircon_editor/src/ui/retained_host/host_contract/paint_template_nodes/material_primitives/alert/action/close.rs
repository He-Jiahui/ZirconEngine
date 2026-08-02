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
    let stroke = alert_close_dot_stroke(frame);
    if stroke <= 0.0 {
        return;
    }
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
            stroke,
        );
        push_alert_close_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            end_y - (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
            stroke,
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

fn alert_close_dot_stroke(frame: &FrameRect) -> f32 {
    ALERT_CLOSE_DOT_EDGE
        .min(frame.width.min(frame.height) * 0.5)
        .max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_close_dot_stroke_fits_short_action_frame() {
        let frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.4,
            height: 0.6,
        };

        assert!(alert_close_dot_stroke(&frame) <= frame.width * 0.5);
        assert!(alert_close_dot_stroke(&frame) <= frame.height * 0.5);
    }
}
