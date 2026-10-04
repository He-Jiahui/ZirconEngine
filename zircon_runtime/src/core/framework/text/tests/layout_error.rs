use super::*;

#[test]
fn font_generation_changed_defers_layout_to_the_next_frame() {
    assert_eq!(
        TextLayoutError::FontGenerationChanged.to_string(),
        "text font database changed; retry layout next frame"
    );
}

#[test]
fn layout_errors_expose_unique_stable_diagnostic_codes_and_catalog_keys() {
    let errors = [
        TextLayoutError::InvalidFontSize,
        TextLayoutError::InvalidLanguage,
        TextLayoutError::FontUnavailable,
        TextLayoutError::FallbackExhausted,
        TextLayoutError::UnsupportedWritingMode,
        TextLayoutError::UnsupportedRenderMode,
        TextLayoutError::ShapingFailed,
        TextLayoutError::BidiInvariant,
        TextLayoutError::RichTextBudgetExceeded,
        TextLayoutError::GeometryTooLarge,
        TextLayoutError::LayoutFailed,
        TextLayoutError::BackendUnavailable,
        TextLayoutError::FontGenerationChanged,
    ];

    let codes = errors.each_ref().map(|error| error.diagnostic_code());
    let keys = errors.each_ref().map(|error| error.message_key());
    for (index, code) in codes.iter().enumerate() {
        assert!(code.starts_with("ZR-TEXT-LAYOUT-"));
        assert!(!codes[..index].contains(code));
    }
    for (index, key) in keys.iter().enumerate() {
        assert!(key.starts_with("text.layout."));
        assert!(!keys[..index].contains(key));
    }
}
