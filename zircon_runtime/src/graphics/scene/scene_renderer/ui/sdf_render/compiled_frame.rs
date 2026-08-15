use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch,
};
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::SdfRunCpuPreparation;

#[derive(Default)]
pub(super) struct PreparedSdfFrameInputs {
    valid: bool,
    viewport_size: UVec2,
    texts: Vec<ScreenSpaceUiTextBatch>,
    cpu_runs: Vec<SdfRunCpuPreparation>,
    native_decoration_texts: Vec<ScreenSpaceUiTextBatch>,
    native_decoration_metrics: Vec<TextDecorationMetrics>,
}

impl PreparedSdfFrameInputs {
    pub(super) fn matches(
        &self,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: &[ScreenSpaceUiTextBatch],
        native_decoration_metrics: &[TextDecorationMetrics],
    ) -> bool {
        self.valid
            && self.viewport_size == viewport_size
            && text_batches_match(&self.texts, texts)
            && self.cpu_runs.as_slice() == cpu_runs
            && text_batches_match(&self.native_decoration_texts, native_decoration_texts)
            && self.native_decoration_metrics.as_slice() == native_decoration_metrics
    }

    pub(super) fn replace(
        &mut self,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: &[ScreenSpaceUiTextBatch],
        native_decoration_metrics: &[TextDecorationMetrics],
    ) {
        self.valid = true;
        self.viewport_size = viewport_size;
        self.texts.clear();
        self.texts.extend_from_slice(texts);
        self.cpu_runs.clear();
        self.cpu_runs.extend_from_slice(cpu_runs);
        self.native_decoration_texts.clear();
        self.native_decoration_texts
            .extend_from_slice(native_decoration_texts);
        self.native_decoration_metrics.clear();
        self.native_decoration_metrics
            .extend_from_slice(native_decoration_metrics);
    }
}

fn text_batches_match(
    prepared: &[ScreenSpaceUiTextBatch],
    current: &[ScreenSpaceUiTextBatch],
) -> bool {
    prepared.len() == current.len()
        && prepared
            .iter()
            .zip(current)
            .all(|(prepared, current)| text_batch_matches(prepared, current))
}

fn text_batch_matches(prepared: &ScreenSpaceUiTextBatch, current: &ScreenSpaceUiTextBatch) -> bool {
    prepared.text == current.text
        && prepared.frame == current.frame
        && prepared.clip_frame == current.clip_frame
        && prepared.source_range == current.source_range
        && prepared.glyph_advances == current.glyph_advances
        && prepared.shaped_glyphs == current.shaped_glyphs
        && glyph_artifact_cache_identity(prepared) == glyph_artifact_cache_identity(current)
        && prepared.color == current.color
        && prepared.font == current.font
        && prepared.font_family == current.font_family
        && prepared.language == current.language
        && prepared.font_weight == current.font_weight
        && prepared.font_size == current.font_size
        && prepared.line_height == current.line_height
        && prepared.text_align == current.text_align
        && prepared.text_direction == current.text_direction
        && prepared.writing_mode == current.writing_mode
        && prepared.wrap == current.wrap
        && prepared.style == current.style
        && prepared.distance_field_mode == current.distance_field_mode
        && prepared.text_effects == current.text_effects
        && prepared.text_decorations == current.text_decorations
        && prepared.text_decoration_baseline == current.text_decoration_baseline
        && prepared.clip_transform == current.clip_transform
}

fn glyph_artifact_cache_identity(
    text: &ScreenSpaceUiTextBatch,
) -> Option<ScreenSpaceUiGlyphArtifactCacheIdentity> {
    text.glyph_artifact_line
        .as_ref()
        .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
}
