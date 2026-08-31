use std::sync::Arc;

use crate::core::framework::text::TextGlyph;
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};

#[derive(Clone, Debug)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiGlyphArtifactLine {
    pub(in crate::graphics::scene::scene_renderer::ui) artifact: Arc<ResolvedTextGlyphArtifact>,
    pub(in crate::graphics::scene::scene_renderer::ui) line_index: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) font_generation: u64,
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_range: std::ops::Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiGlyphArtifactCacheIdentity {
    artifact_address: usize,
    line_address: Option<usize>,
    line_index: usize,
    font_generation: u64,
    glyph_start: usize,
    glyph_end: usize,
}

impl ScreenSpaceUiGlyphArtifactLine {
    pub(in crate::graphics::scene::scene_renderer::ui) fn glyphs(&self) -> Option<&[TextGlyph]> {
        self.artifact
            .lines
            .get(self.line_index)
            .and_then(Option::as_ref)
            .and_then(|line| line.glyphs.get(self.glyph_range.clone()))
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn layout_baseline(&self) -> Option<f32> {
        self.artifact
            .lines
            .get(self.line_index)
            .and_then(Option::as_ref)
            .map(|line| line.layout_line.baseline)
            .filter(|baseline| baseline.is_finite())
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn source_scalar(
        &self,
        glyph: &TextGlyph,
    ) -> char {
        let start = glyph
            .source_range
            .start
            .checked_sub(self.artifact.source_text_origin);
        let end = glyph
            .source_range
            .end
            .checked_sub(self.artifact.source_text_origin);
        start
            .zip(end)
            .and_then(|(start, end)| self.artifact.source_text.get(start..end))
            .and_then(|source| source.chars().next())
            .unwrap_or(' ')
    }

    // Text/layout republishes an immutable artifact Arc after invalidation. The renderer only
    // needs this O(1) identity to invalidate derived atlas and CPU-run caches.
    pub(in crate::graphics::scene::scene_renderer::ui) fn cache_identity(
        &self,
    ) -> ScreenSpaceUiGlyphArtifactCacheIdentity {
        let line_address = self
            .artifact
            .lines
            .get(self.line_index)
            .and_then(Option::as_ref)
            .map(|line| line as *const ResolvedTextGlyphArtifactLine as usize);
        ScreenSpaceUiGlyphArtifactCacheIdentity {
            artifact_address: Arc::as_ptr(&self.artifact) as usize,
            line_address,
            line_index: self.line_index,
            font_generation: self.font_generation,
            glyph_start: self.glyph_range.start,
            glyph_end: self.glyph_range.end,
        }
    }
}
