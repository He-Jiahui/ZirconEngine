use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextLayoutError {
    InvalidFontSize,
    InvalidLanguage,
    FontUnavailable,
    FallbackExhausted,
    UnsupportedWritingMode,
    UnsupportedRenderMode,
    ShapingFailed,
    /// Internal UAX#9 source or glyph-range invariant failed; do not publish reordered text.
    BidiInvariant,
    /// Rich source or visible output exceeded the configured parser representation budget.
    RichTextBudgetExceeded,
    /// Finite or non-finite layout geometry exceeded the active admission policy.
    GeometryTooLarge,
    LayoutFailed,
    BackendUnavailable,
    /// The active font database changed while shaping; retry on a later frame.
    FontGenerationChanged,
}

impl Display for TextLayoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFontSize => {
                formatter.write_str("text font size must be finite and positive")
            }
            Self::InvalidLanguage => formatter.write_str("text language tag is invalid"),
            Self::FontUnavailable => formatter.write_str("requested text font is unavailable"),
            Self::FallbackExhausted => formatter.write_str("text font fallback is exhausted"),
            Self::UnsupportedWritingMode => formatter.write_str("text writing mode is unsupported"),
            Self::UnsupportedRenderMode => formatter.write_str("text render mode is unsupported"),
            Self::ShapingFailed => formatter.write_str("text shaping failed"),
            Self::BidiInvariant => formatter.write_str("text bidi source-range invariant failed"),
            Self::RichTextBudgetExceeded => {
                formatter.write_str("rich text exceeded its parser byte budget")
            }
            Self::GeometryTooLarge => {
                formatter.write_str("text layout geometry exceeds the admitted extent")
            }
            Self::LayoutFailed => formatter.write_str("text layout failed"),
            Self::BackendUnavailable => formatter.write_str("text layout backend is unavailable"),
            Self::FontGenerationChanged => {
                formatter.write_str("text font database changed; retry layout next frame")
            }
        }
    }
}

impl TextLayoutError {
    /// Stable machine-readable identifier for editor, telemetry, and host APIs.
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::InvalidFontSize => "ZR-TEXT-LAYOUT-001",
            Self::InvalidLanguage => "ZR-TEXT-LAYOUT-002",
            Self::FontUnavailable => "ZR-TEXT-LAYOUT-003",
            Self::FallbackExhausted => "ZR-TEXT-LAYOUT-004",
            Self::UnsupportedWritingMode => "ZR-TEXT-LAYOUT-005",
            Self::UnsupportedRenderMode => "ZR-TEXT-LAYOUT-006",
            Self::ShapingFailed => "ZR-TEXT-LAYOUT-007",
            Self::BidiInvariant => "ZR-TEXT-LAYOUT-008",
            Self::RichTextBudgetExceeded => "ZR-TEXT-LAYOUT-012",
            Self::GeometryTooLarge => "ZR-TEXT-LAYOUT-013",
            Self::LayoutFailed => "ZR-TEXT-LAYOUT-009",
            Self::BackendUnavailable => "ZR-TEXT-LAYOUT-010",
            Self::FontGenerationChanged => "ZR-TEXT-LAYOUT-011",
        }
    }

    /// Stable localization catalog key. User-facing text remains outside this enum.
    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::InvalidFontSize => "text.layout.invalid_font_size",
            Self::InvalidLanguage => "text.layout.invalid_language",
            Self::FontUnavailable => "text.layout.font_unavailable",
            Self::FallbackExhausted => "text.layout.fallback_exhausted",
            Self::UnsupportedWritingMode => "text.layout.unsupported_writing_mode",
            Self::UnsupportedRenderMode => "text.layout.unsupported_render_mode",
            Self::ShapingFailed => "text.layout.shaping_failed",
            Self::BidiInvariant => "text.layout.bidi_invariant",
            Self::RichTextBudgetExceeded => "text.layout.rich_text_budget_exceeded",
            Self::GeometryTooLarge => "text.layout.geometry_too_large",
            Self::LayoutFailed => "text.layout.layout_failed",
            Self::BackendUnavailable => "text.layout.backend_unavailable",
            Self::FontGenerationChanged => "text.layout.font_generation_changed",
        }
    }
}

impl Error for TextLayoutError {}

#[cfg(test)]
mod tests {
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

        let codes = errors.map(|error| error.diagnostic_code());
        let keys = errors.map(|error| error.message_key());
        for (index, code) in codes.iter().enumerate() {
            assert!(code.starts_with("ZR-TEXT-LAYOUT-"));
            assert!(!codes[..index].contains(code));
        }
        for (index, key) in keys.iter().enumerate() {
            assert!(key.starts_with("text.layout."));
            assert!(!keys[..index].contains(key));
        }
    }
}
