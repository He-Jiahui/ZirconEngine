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
    let Some(line) = alert_action_line_frame(frame) else {
        return;
    };
    let radius = line.height * 0.5;
    commands.push(HostPaintCommand::quad(
        line,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn alert_action_line_frame(frame: &FrameRect) -> Option<FrameRect> {
    let width = frame.width.max(0.0);
    let height = frame.height.max(0.0);
    let stroke = width.min(height).min(2.0);
    if stroke <= 0.0 {
        return None;
    }
    let inset = 3.0f32.min((width - stroke).max(0.0) * 0.5);
    let line_width = (width - inset * 2.0).max(0.0);
    (line_width > 0.0).then_some(FrameRect {
        x: frame.x + inset,
        y: frame.y + (height - stroke) * 0.5,
        width: line_width,
        height: stroke,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_action_line_stays_inside_short_action_frame() {
        let frame = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.4,
            height: 0.6,
        };
        let line = alert_action_line_frame(&frame).expect("short action has a line");

        assert!(line.x >= frame.x);
        assert!(line.y >= frame.y);
        assert!(line.right() <= frame.right());
        assert!(line.bottom() <= frame.bottom());
    }
}
