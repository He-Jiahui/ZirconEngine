use super::super::super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn drag_overlay_metrics_project_from_shared_host_control_tokens() {
    let mut host = METRICS;
    host.border_width = 1.5;
    host.radius_control = 5.0;
    host.font_body = 12.0;
    host.line_height_ratio = 1.25;
    host.button_pad_x = 14.0;
    host.button_icon_gap = 6.0;
    host.gap_l = 13.0;
    host.row_height = 30.0;
    host.tab_underline_height = 3.0;

    let overlay = drag_overlay_metrics_from_host(host);

    assert_eq!(overlay.border_width, 1.5);
    assert_eq!(overlay.preview_radius, 5.0);
    assert_eq!(overlay.icon_radius, 5.0);
    assert_eq!(overlay.font_size, 12.0);
    assert_eq!(overlay.line_height, 15.0);
    assert_eq!(overlay.icon_left, 14.0);
    assert_eq!(overlay.icon_size, 17.0);
    assert_eq!(overlay.text_left_with_icon, 37.0);
    assert_eq!(overlay.text_right_inset, 14.0);
    assert_eq!(overlay.indicator_thickness, 3.0);
}
