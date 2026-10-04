use crate::text::font::FontDatabaseError;
use crate::text::FontFaceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum BackendFontOperation {
    ResolveVariations,
    LoadFaceBytes,
    ResolveFaceIndex,
}

#[derive(Debug, thiserror::Error)]
/// 这些错误只描述字体后端边界；上层把它们包装成带 source range 的 `DirectShapeError`，再决定是否尝试 alternate backend。
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
