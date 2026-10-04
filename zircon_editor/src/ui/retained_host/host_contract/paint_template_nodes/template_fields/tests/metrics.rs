use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn field_metrics_project_from_host_control_metrics() {
    let host_metrics = HostControlMetrics {
        row_height: 30.0,
        gap_s: 5.0,
        gap_m: 9.0,
        border_width: 2.0,
        font_body: 12.0,
        input_pad: [9.0, 9.0, 3.0, 5.0],
        ..METRICS
    };

    let metrics = workbench_field_metrics_from_host(host_metrics);

    assert_eq!(metrics.font_size, 12.0);
    assert!((metrics.line_height - 14.4).abs() < 0.001);
    assert_eq!(metrics.search_icon_size, 18.0);
    assert_eq!(metrics.search_text_left, 32.0);
    assert_eq!(metrics.search_max_height, 38.0);
    assert_eq!(metrics.stepper_width, 18.0);
    assert_eq!(metrics.stepper_divider_width, 2.0);
    assert_eq!(metrics.stepper_divider_inset_y, 5.0);
    assert_eq!(metrics.stepper_glyph_width, 13.0);
}
