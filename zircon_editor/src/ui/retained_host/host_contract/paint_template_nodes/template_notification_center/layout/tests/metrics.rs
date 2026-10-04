use super::super::super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn notification_metrics_project_from_shared_host_control_tokens() {
    let mut host = METRICS;
    host.radius_control = 5.0;
    host.border_width = 1.5;
    host.font_small = 10.0;
    host.font_body = 12.0;
    host.line_height_ratio = 1.25;
    host.input_pad = [7.0, 7.0, 2.0, 3.0];
    host.button_pad_x = 14.0;
    host.gap_s = 3.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.row_height = 30.0;
    host.selection_indicator_width = 2.0;

    let notification = notification_center_metrics_from_host(host);

    assert_eq!(notification.panel_radius, 8.0);
    assert_eq!(notification.row_radius, 2.0);
    assert_eq!(notification.border_width, 1.5);
    assert_eq!(notification.header_font_size, 12.0);
    assert_eq!(notification.header_line_height, 15.0);
    assert_eq!(notification.message_font_size, 10.0);
    assert_eq!(notification.message_line_height, 12.5);
    assert_eq!(notification.title_font_size, 12.0);
    assert_eq!(notification.title_line_height, 15.0);
    assert_eq!(notification.header_top, 12.0);
    assert_eq!(notification.panel_padding_x, 14.0);
    assert_eq!(notification.row_gap, 6.0);
    assert_eq!(notification.row_height, 45.0);
    assert_eq!(notification.row_inset_x, 9.0);
    assert_eq!(notification.row_top, 39.0);
    assert_eq!(notification.mark_left, 12.0);
    assert_eq!(notification.mark_width, 3.5);
    assert_eq!(notification.text_left, 26.0);
    assert_eq!(notification.text_right_inset, 14.0);
    assert_eq!(notification.title_top, 5.0);
    assert_eq!(notification.message_top, 23.0);
}
