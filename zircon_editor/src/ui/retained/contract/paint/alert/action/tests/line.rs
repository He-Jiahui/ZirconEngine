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
