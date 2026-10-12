use super::*;

#[test]
fn runtime_text_measure_guard_rejects_empty_or_invalid_input() {
    assert!(!should_measure_runtime_text("", 12.0));
    assert!(!should_measure_runtime_text("label", 0.0));
    assert!(!should_measure_runtime_text("label", f32::NAN));
    assert!(should_measure_runtime_text("label", 12.0));
}

#[test]
fn measured_width_is_non_negative() {
    assert_eq!(empty_runtime_text_width(), 0.0);
    assert_eq!(measured_text_width(-4.0), 0.0);
    assert_eq!(measured_text_width(18.5), 18.5);
}

#[test]
fn runtime_style_metrics_fallback_to_resolved_defaults() {
    assert_eq!(resolved_runtime_font_size(13.0), 13.0);
    assert_eq!(
        resolved_runtime_font_size(f32::NAN),
        UiResolvedStyle::DEFAULT_FONT_SIZE
    );
    assert_eq!(resolved_runtime_line_height(13.0, 15.0), 15.0);
    assert_eq!(
        default_runtime_line_height(13.0),
        UiResolvedStyle::default_line_height(13.0)
    );
    assert_eq!(
        resolved_runtime_line_height(13.0, 0.0),
        UiResolvedStyle::default_line_height(13.0)
    );
}
