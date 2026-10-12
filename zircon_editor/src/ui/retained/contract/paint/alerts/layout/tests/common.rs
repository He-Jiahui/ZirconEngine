use super::paint_rect;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn alert_paint_rect_preserves_fractional_geometry_and_degenerate_extents() {
    let aligned = paint_rect(&FrameRect {
        x: 4.4,
        y: 6.6,
        width: 0.4,
        height: -2.0,
    });

    assert_eq!(aligned.x, 4.4);
    assert_eq!(aligned.y, 6.6);
    assert_eq!(aligned.width, 0.0);
    assert_eq!(aligned.height, 0.0);
}
