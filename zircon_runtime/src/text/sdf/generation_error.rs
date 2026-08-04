use std::fmt;

use crate::core::math::UVec2;

/// Typed generation failures are mapped to the normal per-glyph native fallback path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SdfGlyphGenerationError {
    InvalidFaceIndex(u32),
    MissingGlyphOutline(u16),
    EmptyGlyphBounds(u16),
    GenerationPending,
    GenerationBudgetDeferred,
    WorkerPanic,
    InvalidDimensions(UVec2),
    InvalidChannelCount { expected: u8, actual: u8 },
    InvalidOutputLength { expected: usize, actual: usize },
}

impl fmt::Display for SdfGlyphGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFaceIndex(face_index) => {
                write!(formatter, "invalid font face index {face_index}")
            }
            Self::MissingGlyphOutline(glyph_id) => {
                write!(formatter, "glyph {glyph_id} has no outline")
            }
            Self::EmptyGlyphBounds(glyph_id) => {
                write!(formatter, "glyph {glyph_id} has empty bounds")
            }
            Self::GenerationPending => write!(formatter, "distance-field generation is pending"),
            Self::GenerationBudgetDeferred => {
                write!(
                    formatter,
                    "distance-field generation was deferred by frame budget"
                )
            }
            Self::WorkerPanic => write!(formatter, "distance-field generation worker panicked"),
            Self::InvalidDimensions(size) => {
                write!(
                    formatter,
                    "invalid distance-field size {}x{}",
                    size.x, size.y
                )
            }
            Self::InvalidChannelCount { expected, actual } => write!(
                formatter,
                "invalid distance-field channel count: expected {expected}, got {actual}"
            ),
            Self::InvalidOutputLength { expected, actual } => write!(
                formatter,
                "invalid distance-field byte length: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for SdfGlyphGenerationError {}
