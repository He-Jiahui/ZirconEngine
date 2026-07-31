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
            Self::LayoutFailed => formatter.write_str("text layout failed"),
            Self::BackendUnavailable => formatter.write_str("text layout backend is unavailable"),
            Self::FontGenerationChanged => {
                formatter.write_str("text font database changed; retry layout next frame")
            }
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
}
