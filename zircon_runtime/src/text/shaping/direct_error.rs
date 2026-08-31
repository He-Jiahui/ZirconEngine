use crate::text::{FontFaceId, TextRange};

use super::backend_error::BackendShapeError;
use super::bidi::BidiInvariantError;
use super::itemize::ItemizationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum BackendGlyphInvariantKind {
    EmptyOutput,
    InvalidClusterOffset,
    NonFiniteMetrics,
    NonMonotonicClusterOrder,
}

#[derive(Debug, thiserror::Error)]
pub(in crate::text::shaping) enum DirectShapeError {
    #[error("text itemization failed: {0}")]
    Itemization(#[from] ItemizationError),
    #[error("text bidi invariant failed: {0:?}")]
    BidiInvariant(BidiInvariantError),
    #[error("direct shaping backend failed at {range:?}: {source}")]
    Backend {
        range: TextRange,
        #[source]
        source: BackendShapeError,
    },
    #[error("direct shaping source range is invalid: {range:?}")]
    InvalidSourceRange { range: TextRange },
    #[error("direct backend glyph invariant {kind:?} failed for {face:?} at {range:?}")]
    BackendGlyphInvariant {
        face: FontFaceId,
        range: TextRange,
        kind: BackendGlyphInvariantKind,
    },
}

impl DirectShapeError {
    pub(in crate::text::shaping) const fn backend(
        range: TextRange,
        source: BackendShapeError,
    ) -> Self {
        Self::Backend { range, source }
    }

    pub(in crate::text::shaping) const fn backend_glyph_invariant(
        face: FontFaceId,
        range: TextRange,
        kind: BackendGlyphInvariantKind,
    ) -> Self {
        Self::BackendGlyphInvariant { face, range, kind }
    }
}

impl From<BidiInvariantError> for DirectShapeError {
    fn from(error: BidiInvariantError) -> Self {
        Self::BidiInvariant(error)
    }
}

pub(in crate::text::shaping) fn validate_backend_glyphs<T>(
    glyphs: &[T],
    text: &str,
    source_offset: impl Fn(&T) -> usize,
    metrics_are_finite: impl Fn(&T) -> bool,
) -> Result<(), BackendGlyphInvariantKind> {
    if glyphs.is_empty() {
        return Err(BackendGlyphInvariantKind::EmptyOutput);
    }
    if glyphs.iter().any(|glyph| {
        let offset = source_offset(glyph);
        offset >= text.len() || !text.is_char_boundary(offset)
    }) {
        return Err(BackendGlyphInvariantKind::InvalidClusterOffset);
    }
    if glyphs.iter().any(|glyph| !metrics_are_finite(glyph)) {
        return Err(BackendGlyphInvariantKind::NonFiniteMetrics);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::text::TextRange;
    use crate::text::shaping::backend_error::BackendShapeError;

    use super::DirectShapeError;

    #[test]
    fn backend_failures_retain_the_itemized_source_range() {
        let range = TextRange { start: 4, end: 9 };
        let face = crate::text::FontFaceId(7);

        let error = DirectShapeError::backend(
            range,
            BackendShapeError::FaceParseFailed {
                face,
                face_index: 2,
            },
        );

        assert!(matches!(
            error,
            DirectShapeError::Backend {
                range: retained,
                source: BackendShapeError::FaceParseFailed {
                    face: retained_face,
                    face_index: 2,
                },
            } if retained == range && retained_face == face
        ));
    }
}
