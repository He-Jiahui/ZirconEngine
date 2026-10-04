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
/// 直接后端错误保留 itemization/bidi、字体加载和 glyph 校验的阶段信息；回退分类器依赖这些变体而非错误字符串。
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

/// 验证后端 glyph 非空、cluster 起点位于文本内且为 UTF-8 字符边界、度量有限；失败保留具体类别供回执分类。
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
#[path = "tests/direct_error.rs"]
mod tests;
