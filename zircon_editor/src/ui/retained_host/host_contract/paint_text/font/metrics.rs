use zircon_runtime_interface::ui::surface::UiResolvedStyle;

const EMPTY_TEXT_MEASURE_WIDTH_PX: f32 = 0.0;

pub(super) fn should_measure_runtime_text(text: &str, font_size: f32) -> bool {
    !text.is_empty() && valid_runtime_font_size(font_size)
}

pub(super) fn empty_runtime_text_width() -> f32 {
    EMPTY_TEXT_MEASURE_WIDTH_PX
}

pub(super) fn measured_text_width(width: f32) -> f32 {
    width.max(EMPTY_TEXT_MEASURE_WIDTH_PX)
}

pub(super) fn resolved_runtime_font_size(font_size: f32) -> f32 {
    if valid_runtime_font_size(font_size) {
        font_size
    } else {
        UiResolvedStyle::DEFAULT_FONT_SIZE
    }
}

pub(super) fn resolved_runtime_line_height(font_size: f32, line_height: f32) -> f32 {
    if line_height.is_finite() && line_height > EMPTY_TEXT_MEASURE_WIDTH_PX {
        line_height
    } else {
        default_runtime_line_height(font_size)
    }
}

pub(super) fn default_runtime_line_height(font_size: f32) -> f32 {
    UiResolvedStyle::default_line_height(font_size)
}

fn valid_runtime_font_size(font_size: f32) -> bool {
    font_size.is_finite() && font_size > EMPTY_TEXT_MEASURE_WIDTH_PX
}

#[cfg(test)]
mod tests {
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
}
