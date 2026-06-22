use super::super::{ripple_diameter, ripple_rect, TemplatePaneNodeData, RIPPLE_DIAMETER_EXPANSION};
use super::support::frame;

#[test]
fn ripple_diameter_matches_slint_width_based_source_contract() {
    let rect = frame(4.0, 8.0, 20.0, 80.0);
    let ripple = ripple_rect(&TemplatePaneNodeData::default(), &rect);
    let expected = 20.0 * RIPPLE_DIAMETER_EXPANSION;

    assert_eq!(ripple.width, expected);
    assert_eq!(ripple.height, expected);
}

#[test]
fn ripple_origin_preserves_zero_press_coordinates() {
    let node = TemplatePaneNodeData {
        ripple_pressed_x: 0.0,
        ripple_pressed_y: 0.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = frame(10.0, 20.0, 24.0, 24.0);
    let ripple = ripple_rect(&node, &rect);
    let radius = ripple_diameter(&rect) * 0.5;

    assert_eq!(ripple.x + radius, rect.x);
    assert_eq!(ripple.y + radius, rect.y);
}
