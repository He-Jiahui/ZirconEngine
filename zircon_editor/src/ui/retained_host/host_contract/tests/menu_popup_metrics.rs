use super::*;

#[test]
fn visible_row_range_keeps_only_strict_viewport_intersections() {
    assert_eq!(menu_popup_visible_row_range(32, 60.0, 60.0, 0.0), 2..4);
    assert_eq!(menu_popup_visible_row_range(32, 60.0, 28.0, 0.0), 1..3);
    assert_eq!(menu_popup_visible_row_range(32, 60.0, 0.0, 6.0), 0..2);
}

#[test]
fn visible_row_range_rejects_invalid_viewports_and_offsets() {
    assert_eq!(menu_popup_visible_row_range(0, 60.0, 0.0, 0.0), 0..0);
    assert_eq!(menu_popup_visible_row_range(8, 0.0, 0.0, 0.0), 0..0);
    assert_eq!(menu_popup_visible_row_range(8, 60.0, f32::NAN, 0.0), 0..0);
}
