use crate::text::FontFaceId;
use crate::text::font::FontDatabaseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum BackendFontOperation {
    ResolveVariations,
    LoadFaceBytes,
    ResolveFaceIndex,
}

#[derive(Debug, thiserror::Error)]
pub(in crate::text::shaping) enum BackendShapeError {
    #[error("font backend {operation:?} failed for {face:?}: {source}")]
    FontDatabase {
        operation: BackendFontOperation,
        face: FontFaceId,
        #[source]
        source: FontDatabaseError,
    },
    #[error("font backend could not parse face {face:?} at collection index {face_index}")]
    FaceParseFailed { face: FontFaceId, face_index: u32 },
    #[error("font backend returned no glyphs for non-empty input using {face:?}")]
    EmptyGlyphOutput { face: FontFaceId },
}

impl BackendShapeError {
    pub(in crate::text::shaping) fn font_database(
        operation: BackendFontOperation,
        face: FontFaceId,
        source: FontDatabaseError,
    ) -> Self {
        Self::FontDatabase {
            operation,
            face,
            source,
        }
    }
}
