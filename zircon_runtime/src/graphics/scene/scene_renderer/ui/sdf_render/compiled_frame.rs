use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch,
};
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::SdfRunCpuPreparation;

use super::super::text::ScreenSpaceUiTextFrameProductGeneration;

#[derive(Default)]
pub(super) struct PreparedSdfFrameInputs {
    valid: bool,
    retained_frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    viewport_size: UVec2,
    texts: Vec<ScreenSpaceUiTextBatch>,
    cpu_runs: Vec<SdfRunCpuPreparation>,
    native_decoration_texts: Vec<ScreenSpaceUiTextBatch>,
    native_decoration_metrics: Vec<TextDecorationMetrics>,
}

impl PreparedSdfFrameInputs {
    #[cfg(test)]
    pub(super) fn matches(
        &self,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: &[ScreenSpaceUiTextBatch],
        native_decoration_metrics: &[TextDecorationMetrics],
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool {
        self.matches_iter(
            viewport_size,
            texts.iter(),
            cpu_runs,
            native_decoration_texts.iter(),
            native_decoration_metrics,
            retained_generation,
        )
    }

    pub(super) fn matches_iter<'a, Texts, NativeTexts>(
        &self,
        viewport_size: UVec2,
        texts: Texts,
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: NativeTexts,
        native_decoration_metrics: &[TextDecorationMetrics],
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool
    where
        Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        if self.valid
            && retained_generation.is_some()
            && self.retained_frame_generation == retained_generation
            && self.viewport_size == viewport_size
        {
            return true;
        }
        self.valid
            && self.viewport_size == viewport_size
            && text_batches_match_iter(&self.texts, texts)
            && self.cpu_runs.as_slice() == cpu_runs
            && text_batches_match_iter(&self.native_decoration_texts, native_decoration_texts)
            && self.native_decoration_metrics.as_slice() == native_decoration_metrics
    }

    #[cfg(test)]
    pub(super) fn replace(
        &mut self,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: &[ScreenSpaceUiTextBatch],
        native_decoration_metrics: &[TextDecorationMetrics],
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) {
        self.replace_iter(
            viewport_size,
            texts.iter(),
            cpu_runs,
            native_decoration_texts.iter(),
            native_decoration_metrics,
            retained_generation,
        );
    }

    pub(super) fn replace_iter<'a, Texts, NativeTexts>(
        &mut self,
        viewport_size: UVec2,
        texts: Texts,
        cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_texts: NativeTexts,
        native_decoration_metrics: &[TextDecorationMetrics],
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) where
        Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        self.valid = true;
        self.retained_frame_generation = retained_generation;
        self.viewport_size = viewport_size;
        self.texts.clear();
        self.cpu_runs.clear();
        self.native_decoration_texts.clear();
        self.native_decoration_metrics.clear();
        if retained_generation.is_none() {
            self.texts.extend(texts.into_iter().cloned());
            self.cpu_runs.extend_from_slice(cpu_runs);
            self.native_decoration_texts
                .extend(native_decoration_texts.into_iter().cloned());
            self.native_decoration_metrics
                .extend_from_slice(native_decoration_metrics);
        }
    }
}

fn text_batches_match_iter<'a, Texts>(prepared: &[ScreenSpaceUiTextBatch], current: Texts) -> bool
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    let mut current = current.into_iter();
    prepared.iter().all(|prepared| {
        current
            .next()
            .is_some_and(|current| text_batch_matches(prepared, current))
    }) && current.next().is_none()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_generation_hit_still_requires_the_published_viewport() {
        let mut generation_counter = 0;
        let generation = ScreenSpaceUiTextFrameProductGeneration::next(&mut generation_counter);
        let mut prepared = PreparedSdfFrameInputs::default();
        prepared.replace(UVec2::new(800, 600), &[], &[], &[], &[], Some(generation));

        assert!(prepared.matches(UVec2::new(800, 600), &[], &[], &[], &[], Some(generation),));
        assert!(!prepared.matches(UVec2::new(801, 600), &[], &[], &[], &[], Some(generation),));
    }

    #[test]
    fn retained_generation_replaces_owned_fallback_snapshots_with_generation_authority() {
        let mut generation_counter = 0;
        let generation = ScreenSpaceUiTextFrameProductGeneration::next(&mut generation_counter);
        let mut prepared = PreparedSdfFrameInputs::default();
        prepared.replace(
            UVec2::new(800, 600),
            &[],
            &[SdfRunCpuPreparation::default()],
            &[],
            &[TextDecorationMetrics::default()],
            None,
        );
        assert_eq!(prepared.cpu_runs.len(), 1);
        assert_eq!(prepared.native_decoration_metrics.len(), 1);

        prepared.replace_iter(
            UVec2::new(800, 600),
            std::iter::empty::<&ScreenSpaceUiTextBatch>(),
            &[],
            std::iter::empty::<&ScreenSpaceUiTextBatch>(),
            &[],
            Some(generation),
        );

        assert!(prepared.texts.is_empty());
        assert!(prepared.cpu_runs.is_empty());
        assert!(prepared.native_decoration_texts.is_empty());
        assert!(prepared.native_decoration_metrics.is_empty());
        assert!(prepared.matches_iter(
            UVec2::new(800, 600),
            std::iter::empty::<&ScreenSpaceUiTextBatch>(),
            &[],
            std::iter::empty::<&ScreenSpaceUiTextBatch>(),
            &[],
            Some(generation),
        ));
    }
}
