use super::{text_layout_error_layout, MIN_TEXT_FONT_SIZE};
use crate::core::framework::text::TextLayoutError;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection};

#[test]
fn failure_layout_rejects_non_finite_and_over_budget_metrics() {
    let style = UiResolvedStyle::default();
    let mut provider = SharedTextLayoutSession::new();
    let invalid = text_layout_error_layout(
        &style,
        UiTextDirection::LeftToRight,
        f32::MAX,
        f32::INFINITY,
        4,
        &TextLayoutError::InvalidFontSize,
        &mut provider,
    );

    assert_eq!(invalid.font_size, MIN_TEXT_FONT_SIZE);
    assert_eq!(invalid.line_height, MIN_TEXT_FONT_SIZE);
    assert_eq!(invalid.measured_height, MIN_TEXT_FONT_SIZE);
    assert!(invalid.font_size.is_finite());
    assert!(invalid.line_height.is_finite());

    let valid = text_layout_error_layout(
        &style,
        UiTextDirection::LeftToRight,
        12.0,
        18.0,
        4,
        &TextLayoutError::LayoutFailed,
        &mut provider,
    );
    assert_eq!(valid.font_size, 12.0);
    assert_eq!(valid.line_height, 18.0);
    assert_eq!(valid.measured_height, 18.0);
}
