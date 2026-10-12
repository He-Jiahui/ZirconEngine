use super::*;

#[test]
fn alert_action_stays_inside_tight_alert_bounds() {
    let alert = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.4,
        height: 0.6,
    };
    let frame = alert_action_frame(&alert);

    assert!(frame.x >= alert.x);
    assert!(frame.y >= alert.y);
    assert!(frame.right() <= alert.right());
    assert!(frame.bottom() <= alert.bottom());
}
