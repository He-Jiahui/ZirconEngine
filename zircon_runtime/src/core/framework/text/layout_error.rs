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
        }
    }
}

impl Error for TextLayoutError {}
