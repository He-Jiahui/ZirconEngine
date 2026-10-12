use super::*;

#[test]
fn alert_icon_and_mark_stay_inside_tight_alert_bounds() {
    let alert = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.4,
        height: 0.6,
    };
    let icon = alert_icon_frame(&alert);
    let mark = alert_icon_mark_frame(&icon);

    for frame in [icon, mark] {
        assert!(frame.x >= alert.x);
        assert!(frame.y >= alert.y);
        assert!(frame.right() <= alert.right());
        assert!(frame.bottom() <= alert.bottom());
    }
}
