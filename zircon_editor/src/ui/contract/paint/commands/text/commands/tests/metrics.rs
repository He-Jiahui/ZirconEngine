use super::{resolved_font_size, resolved_line_height, MIN_TEXT_METRIC_PX};

#[test]
fn text_metrics_keep_runtime_values_above_minimum() {
    assert_eq!(resolved_font_size(12.0), 12.0);
    assert_eq!(resolved_line_height(12.0, 15.0), 15.0);
}

#[test]
fn text_metrics_clamp_empty_runtime_values_to_minimum() {
    assert_eq!(resolved_font_size(0.0), MIN_TEXT_METRIC_PX);
    assert_eq!(resolved_line_height(0.0, 0.0), MIN_TEXT_METRIC_PX);
}

#[test]
fn text_metrics_do_not_let_line_height_drop_below_font_size() {
    assert_eq!(resolved_line_height(12.0, 10.0), 12.0);
}
