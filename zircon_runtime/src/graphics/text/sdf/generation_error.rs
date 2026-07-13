use std::fmt;

use crate::core::math::UVec2;

/// Typed generation failures are mapped to the normal per-glyph native fallback path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SdfGlyphGenerationError {
    InvalidFaceIndex(u32),
    MissingGlyphOutline(u16),
    EmptyGlyphBounds(u16),
    InvalidDimensions(UVec2),
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
            Self::InvalidDimensions(size) => {
                write!(
                    formatter,
                    "invalid distance-field size {}x{}",
                    size.x, size.y
                )
            }
            Self::InvalidOutputLength { expected, actual } => write!(
                formatter,
                "invalid distance-field byte length: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for SdfGlyphGenerationError {}
