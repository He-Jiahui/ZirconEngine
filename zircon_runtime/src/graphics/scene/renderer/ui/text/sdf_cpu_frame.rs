use crate::asset::ProjectAssetManager;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::SdfRunCpuPreparation;
use crate::text::TextRenderState;
use std::ops::Range;

use super::{ScreenSpaceUiTextFrameProduct, ScreenSpaceUiTextFrameProductGeneration};

#[derive(Default)]
pub(super) struct SdfTextCpuFrame {
    valid: bool,
    retained_frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    prepared_sdf_texts: Vec<PreparedSdfCpuText>,
    prepared_native_texts: Vec<PreparedSdfCpuText>,
    sdf_runs: Vec<SdfRunCpuPreparation>,
    native_decoration_metrics: Vec<TextDecorationMetrics>,
    segment_products: Vec<SdfTextCpuSegmentProduct>,
    sdf_run_ranges: Vec<Range<usize>>,
    native_metric_ranges: Vec<Range<usize>>,
    last_segment_visit_count: usize,
    last_sdf_run_visit_count: usize,
    last_native_metric_visit_count: usize,
}

struct SdfTextCpuSegmentProduct {
    sdf_runs: Vec<SdfRunCpuPreparation>,
    native_decoration_metrics: Vec<TextDecorationMetrics>,
}

#[derive(Clone)]
struct PreparedSdfCpuText {
    text: String,
    glyph_advances: Vec<f32>,
    shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    glyph_artifact_identity: Option<ScreenSpaceUiGlyphArtifactCacheIdentity>,
    font: Option<String>,
    font_family: Option<String>,
    language: Option<String>,
    font_weight: u16,
    font_size: f32,
    writing_mode: zircon_runtime_interface::ui::surface::UiTextWritingMode,
}

impl SdfTextCpuFrame {
    pub(super) fn prepare(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) -> bool {
        self.prepare_with_retained_generation(
            sdf_texts,
            native_texts,
            text_state,
            asset_manager,
            None,
        )
    }

    pub(super) fn prepare_retained(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> bool {
        self.prepare_with_retained_generation(
            sdf_texts,
            native_texts,
            text_state,
            asset_manager,
            Some(generation),
        )
    }

    pub(super) fn prepare_retained_segments<'a, SdfSegments, NativeSegments>(
        &mut self,
        sdf_segments: SdfSegments,
        native_segments: NativeSegments,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> bool
    where
        SdfSegments: Clone + Iterator<Item = &'a [ScreenSpaceUiTextBatch]>,
        NativeSegments: Clone + Iterator<Item = &'a [ScreenSpaceUiTextBatch]>,
    {
        self.prepare_with_retained_text_iter(
            sdf_segments.flat_map(|segment| segment.iter()),
            native_segments.flat_map(|segment| segment.iter()),
            text_state,
            asset_manager,
            Some(generation),
        )
    }

    pub(super) fn prepare_retained_frame(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) -> bool {
        if self.valid && self.retained_frame_generation == Some(frame.generation()) {
            self.record_segment_profile(0, 0, 0);
            return true;
        }
        let journal = frame.change_journal();
        let local_topology_matches = self
            .segment_products
            .len()
            .checked_sub(journal.truncated_segment_count())
            .and_then(|retained| retained.checked_add(journal.appended_segment_count()))
            .is_some_and(|current| current == frame.segment_products().len());
        let local = !journal.is_full_rebuild()
            && journal.base_generation() == self.retained_frame_generation
            && journal.current_generation() == frame.generation()
            && local_topology_matches
            && self.sdf_run_ranges.len() == self.segment_products.len()
            && self.native_metric_ranges.len() == self.segment_products.len();
        if !local {
            self.rebuild_retained_frame(frame, text_state, asset_manager);
            return false;
        }

        let retained_segment_count = self
            .segment_products
            .len()
            .saturating_sub(journal.truncated_segment_count());
        if journal
            .changed_segment_indices()
            .iter()
            .any(|&index| index >= retained_segment_count)
        {
            self.rebuild_retained_frame(frame, text_state, asset_manager);
            return false;
        }

        let mut sdf_run_visit_count = 0_usize;
        let mut native_metric_visit_count = 0_usize;
        for &index in journal.changed_segment_indices() {
            let replacement = build_cpu_segment_product(
                &frame.segment_products()[index],
                text_state,
                asset_manager,
            );
            sdf_run_visit_count = sdf_run_visit_count.saturating_add(replacement.sdf_runs.len());
            native_metric_visit_count = native_metric_visit_count
                .saturating_add(replacement.native_decoration_metrics.len());
            replace_segment_output(
                &mut self.sdf_runs,
                &mut self.sdf_run_ranges,
                index,
                &replacement.sdf_runs,
            );
            replace_segment_output(
                &mut self.native_decoration_metrics,
                &mut self.native_metric_ranges,
                index,
                &replacement.native_decoration_metrics,
            );
            self.segment_products[index] = replacement;
        }

        truncate_segment_outputs(
            &mut self.sdf_runs,
            &mut self.sdf_run_ranges,
            retained_segment_count,
        );
        truncate_segment_outputs(
            &mut self.native_decoration_metrics,
            &mut self.native_metric_ranges,
            retained_segment_count,
        );
        self.segment_products.truncate(retained_segment_count);
        for segment in &frame.segment_products()[retained_segment_count..] {
            let product = build_cpu_segment_product(segment, text_state, asset_manager);
            sdf_run_visit_count = sdf_run_visit_count.saturating_add(product.sdf_runs.len());
            native_metric_visit_count =
                native_metric_visit_count.saturating_add(product.native_decoration_metrics.len());
            append_segment_output(
                &mut self.sdf_runs,
                &mut self.sdf_run_ranges,
                &product.sdf_runs,
            );
            append_segment_output(
                &mut self.native_decoration_metrics,
                &mut self.native_metric_ranges,
                &product.native_decoration_metrics,
            );
            self.segment_products.push(product);
        }
        self.prepared_sdf_texts.clear();
        self.prepared_native_texts.clear();
        self.retained_frame_generation = Some(frame.generation());
        self.valid = true;
        self.record_segment_profile(
            journal
                .changed_segment_indices()
                .len()
                .saturating_add(journal.appended_segment_count())
                .saturating_add(journal.truncated_segment_count()),
            sdf_run_visit_count,
            native_metric_visit_count,
        );
        false
    }

    fn prepare_with_retained_generation(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool {
        self.prepare_with_retained_text_iter(
            sdf_texts.iter(),
            native_texts.iter(),
            text_state,
            asset_manager,
            retained_generation,
        )
    }

    fn prepare_with_retained_text_iter<'a, SdfTexts, NativeTexts>(
        &mut self,
        sdf_texts: SdfTexts,
        native_texts: NativeTexts,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool
    where
        SdfTexts: Clone + Iterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: Clone + Iterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        if self.valid
            && retained_generation.is_some()
            && self.retained_frame_generation == retained_generation
        {
            return true;
        }
        if self.matches_iter(sdf_texts.clone(), native_texts.clone()) {
            self.retained_frame_generation = retained_generation;
            return true;
        }

        text_state.prepare_sdf_runs_cpu_iter_into(
            sdf_texts.clone(),
            asset_manager,
            &mut self.sdf_runs,
        );
        text_state.prepare_sdf_decoration_metrics_iter_into(
            native_texts.clone(),
            asset_manager,
            &mut self.native_decoration_metrics,
        );
        replace_prepared_texts_iter(&mut self.prepared_sdf_texts, sdf_texts);
        replace_prepared_texts_iter(&mut self.prepared_native_texts, native_texts);
        self.segment_products.clear();
        self.sdf_run_ranges.clear();
        self.native_metric_ranges.clear();
        self.retained_frame_generation = retained_generation;
        self.valid = true;
        false
    }

    pub(super) fn outputs_mut(
        &mut self,
    ) -> (
        &mut Vec<SdfRunCpuPreparation>,
        &mut Vec<TextDecorationMetrics>,
    ) {
        (&mut self.sdf_runs, &mut self.native_decoration_metrics)
    }

    pub(super) fn outputs(&self) -> (&[SdfRunCpuPreparation], &[TextDecorationMetrics]) {
        (&self.sdf_runs, &self.native_decoration_metrics)
    }

    pub(super) fn invalidate(&mut self) {
        self.valid = false;
        self.retained_frame_generation = None;
        self.segment_products.clear();
        self.sdf_run_ranges.clear();
        self.native_metric_ranges.clear();
    }

    fn matches_iter<'a, SdfTexts, NativeTexts>(
        &self,
        sdf_texts: SdfTexts,
        native_texts: NativeTexts,
    ) -> bool
    where
        SdfTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        self.valid
            && text_cpu_inputs_match_iter(&self.prepared_sdf_texts, sdf_texts)
            && text_cpu_inputs_match_iter(&self.prepared_native_texts, native_texts)
    }

    fn rebuild_retained_frame(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) {
        self.prepared_sdf_texts.clear();
        self.prepared_native_texts.clear();
        self.segment_products.clear();
        self.sdf_runs.clear();
        self.native_decoration_metrics.clear();
        self.sdf_run_ranges.clear();
        self.native_metric_ranges.clear();
        for segment in frame.segment_products() {
            let product = build_cpu_segment_product(segment, text_state, asset_manager);
            append_segment_output(
                &mut self.sdf_runs,
                &mut self.sdf_run_ranges,
                &product.sdf_runs,
            );
            append_segment_output(
                &mut self.native_decoration_metrics,
                &mut self.native_metric_ranges,
                &product.native_decoration_metrics,
            );
            self.segment_products.push(product);
        }
        self.retained_frame_generation = Some(frame.generation());
        self.valid = true;
        self.record_segment_profile(
            frame.segment_products().len(),
            self.sdf_runs.len(),
            self.native_decoration_metrics.len(),
        );
    }

    fn record_segment_profile(
        &mut self,
        segment_visit_count: usize,
        sdf_run_visit_count: usize,
        native_metric_visit_count: usize,
    ) {
        self.last_segment_visit_count = segment_visit_count;
        self.last_sdf_run_visit_count = sdf_run_visit_count;
        self.last_native_metric_visit_count = native_metric_visit_count;
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui_text.sdf_cpu.segment_visit_count",
                    segment_visit_count as f64,
                ),
                (
                    "ui_text.sdf_cpu.run_visit_count",
                    sdf_run_visit_count as f64,
                ),
                (
                    "ui_text.sdf_cpu.native_metric_visit_count",
                    native_metric_visit_count as f64,
                ),
            ],
        );
    }

    #[cfg(test)]
    fn segment_visit_report(&self) -> (usize, usize, usize) {
        (
            self.last_segment_visit_count,
            self.last_sdf_run_visit_count,
            self.last_native_metric_visit_count,
        )
    }
}

fn build_cpu_segment_product(
    segment: &super::ScreenSpaceUiTextSegmentProduct,
    text_state: &mut TextRenderState,
    asset_manager: &ProjectAssetManager,
) -> SdfTextCpuSegmentProduct {
    let mut sdf_runs = Vec::new();
    text_state.prepare_sdf_runs_cpu_iter_into(
        segment.sdf_texts().iter(),
        asset_manager,
        &mut sdf_runs,
    );
    let mut native_decoration_metrics = Vec::new();
    text_state.prepare_sdf_decoration_metrics_iter_into(
        segment.native_texts().iter(),
        asset_manager,
        &mut native_decoration_metrics,
    );
    SdfTextCpuSegmentProduct {
        sdf_runs,
        native_decoration_metrics,
    }
}

fn replace_segment_output<T: Clone>(
    output: &mut Vec<T>,
    ranges: &mut [Range<usize>],
    index: usize,
    replacement: &[T],
) {
    let previous = ranges[index].clone();
    let previous_len = previous.len();
    output.splice(previous.clone(), replacement.iter().cloned());
    ranges[index] = previous.start..previous.start.saturating_add(replacement.len());
    shift_ranges(&mut ranges[index + 1..], previous_len, replacement.len());
}

fn append_segment_output<T: Clone>(
    output: &mut Vec<T>,
    ranges: &mut Vec<Range<usize>>,
    segment: &[T],
) {
    let start = output.len();
    output.extend_from_slice(segment);
    ranges.push(start..output.len());
}

fn truncate_segment_outputs<T>(
    output: &mut Vec<T>,
    ranges: &mut Vec<Range<usize>>,
    retained_segment_count: usize,
) {
    let truncate_start = ranges
        .get(retained_segment_count)
        .map_or(output.len(), |range| range.start);
    output.truncate(truncate_start);
    ranges.truncate(retained_segment_count);
}

fn shift_ranges(ranges: &mut [Range<usize>], old_len: usize, new_len: usize) {
    if new_len >= old_len {
        let delta = new_len - old_len;
        for range in ranges {
            range.start = range.start.saturating_add(delta);
            range.end = range.end.saturating_add(delta);
        }
    } else {
        let delta = old_len - new_len;
        for range in ranges {
            range.start = range.start.saturating_sub(delta);
            range.end = range.end.saturating_sub(delta);
        }
    }
}

fn replace_prepared_texts_iter<'a, Texts>(prepared: &mut Vec<PreparedSdfCpuText>, texts: Texts)
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    prepared.clear();
    prepared.extend(texts.into_iter().map(PreparedSdfCpuText::from));
}

fn text_cpu_inputs_match_iter<'a, Texts>(prepared: &[PreparedSdfCpuText], texts: Texts) -> bool
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    let mut texts = texts.into_iter();
    prepared
        .iter()
        .all(|prepared| texts.next().is_some_and(|text| prepared.matches(text)))
        && texts.next().is_none()
}

impl From<&ScreenSpaceUiTextBatch> for PreparedSdfCpuText {
    fn from(text: &ScreenSpaceUiTextBatch) -> Self {
        Self {
            text: text.text.clone(),
            glyph_advances: text.glyph_advances.clone(),
            shaped_glyphs: text.shaped_glyphs.clone(),
            glyph_artifact_identity: text
                .glyph_artifact_line
                .as_ref()
                .map(ScreenSpaceUiGlyphArtifactLine::cache_identity),
            font: text.font.clone(),
            font_family: text.font_family.clone(),
            language: text.language.clone(),
            font_weight: text.font_weight,
            font_size: text.font_size,
            writing_mode: text.writing_mode,
        }
    }
}

impl PreparedSdfCpuText {
    fn matches(&self, text: &ScreenSpaceUiTextBatch) -> bool {
        self.text == text.text
            && self.glyph_advances == text.glyph_advances
            && self.shaped_glyphs == text.shaped_glyphs
            && self.glyph_artifact_identity
                == text
                    .glyph_artifact_line
                    .as_ref()
                    .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
            && self.font == text.font
            && self.font_family == text.font_family
            && self.language == text.language
            && self.font_weight == text.font_weight
            && self.font_size == text.font_size
            && self.writing_mode == text.writing_mode
    }
}

#[cfg(test)]
#[path = "tests/sdf_cpu_frame.rs"]
mod tests;
