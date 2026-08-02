use super::super::render::ScreenSpaceUiTextBatch;
use super::super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;
use super::super::sdf_atlas::{SdfAtlasAllocationFailureReason, SdfAtlasRun};
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::{SdfGlyphGenerationError, SdfRunCpuPreparation};
use crate::text::shaping::resolve_bidi_base_direction;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiTextAlign, UiTextDirection, UiTextWrap, UiTextWritingMode,
};

const MIN_NATIVE_OVERLAY_WIDTH: f32 = 1.0;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextSdfFallbackReport {
    pub(super) fallback_text_batch_count: usize,
    pub(super) whole_batch_fallback_text_batch_count: usize,
    pub(super) fallback_native_overlay_batch_count: usize,
    pub(super) mixed_overlay_unsupported_text_batch_count: usize,
    pub(super) mixed_overlay_empty_span_text_batch_count: usize,
    pub(super) mixed_overlay_missing_advances_text_batch_count: usize,
    pub(super) mixed_overlay_unsupported_writing_mode_text_batch_count: usize,
    pub(super) mixed_overlay_unsupported_text_direction_text_batch_count: usize,
    pub(super) mixed_overlay_unsupported_wrap_text_batch_count: usize,
    pub(super) mixed_overlay_unsupported_justify_text_batch_count: usize,
    pub(super) mixed_overlay_glyph_advance_mismatch_text_batch_count: usize,
    pub(super) mixed_overlay_invalid_span_text_batch_count: usize,
    pub(super) fallback_glyph_count: usize,
    pub(super) fallback_span_count: usize,
    pub(super) fallback_source_byte_count: usize,
    pub(super) page_limit_glyph_count: usize,
    pub(super) oversized_glyph_count: usize,
    pub(super) page_limit_span_count: usize,
    pub(super) oversized_span_count: usize,
    pub(super) page_limit_source_byte_count: usize,
    pub(super) oversized_source_byte_count: usize,
}

impl ScreenSpaceUiTextSdfFallbackReport {
    pub(super) fn has_whole_batch_fallbacks(&self) -> bool {
        self.whole_batch_fallback_text_batch_count > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SdfAtlasGlyphFallbackSpan {
    start_glyph_index: usize,
    glyph_count: usize,
    start_byte_index: usize,
    end_byte_index: usize,
    reason: SdfGlyphFallbackReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SdfGlyphFallbackReason {
    Allocation(SdfAtlasAllocationFailureReason),
    Generation(SdfGlyphGenerationError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MixedNativeOverlayUnsupportedReason {
    EmptyFallbackSpans,
    MissingGlyphAdvances,
    UnsupportedWritingMode,
    UnsupportedTextDirection,
    UnsupportedWrap,
    UnsupportedJustify,
    GlyphAdvanceCountMismatch,
    InvalidFallbackSpan,
}

pub(super) fn apply_sdf_atlas_fallbacks(
    native_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    atlas_runs: &[SdfAtlasRun],
    glyph_advances_by_run: &[Vec<f32>],
) -> ScreenSpaceUiTextSdfFallbackReport {
    if !sdf_runs_need_fallback(sdf_texts, atlas_runs) {
        return ScreenSpaceUiTextSdfFallbackReport::default();
    }
    apply_sdf_atlas_fallbacks_internal(native_texts, sdf_texts, atlas_runs, |index| {
        glyph_advances_by_run.get(index).map(Vec::as_slice)
    })
    .report
}

pub(super) fn apply_sdf_atlas_fallbacks_with_cpu_runs(
    native_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    atlas_runs: &[SdfAtlasRun],
    cpu_runs: &mut Vec<SdfRunCpuPreparation>,
    native_decoration_metrics: &mut Vec<TextDecorationMetrics>,
) -> ScreenSpaceUiTextSdfFallbackReport {
    if !sdf_runs_need_fallback(sdf_texts, atlas_runs) {
        return ScreenSpaceUiTextSdfFallbackReport::default();
    }
    let result = apply_sdf_atlas_fallbacks_internal(native_texts, sdf_texts, atlas_runs, |index| {
        cpu_runs.get(index).map(|run| run.glyph_advances.as_slice())
    });
    native_decoration_metrics.extend(result.native_fallback_run_indices.iter().map(|index| {
        cpu_runs
            .get(*index)
            .map(|run| run.decoration_metrics)
            .unwrap_or_default()
    }));
    let mut retained_indices = result.retained_sdf_run_indices.into_iter().peekable();
    let pending_runs = std::mem::take(cpu_runs);
    *cpu_runs = pending_runs
        .into_iter()
        .enumerate()
        .filter_map(|(index, run)| {
            (retained_indices.peek() == Some(&index)).then(|| {
                let _ = retained_indices.next();
                run
            })
        })
        .collect();
    result.report
}

fn sdf_runs_need_fallback(
    sdf_texts: &[ScreenSpaceUiTextBatch],
    atlas_runs: &[SdfAtlasRun],
) -> bool {
    sdf_texts.len() != atlas_runs.len() || atlas_runs.iter().any(SdfAtlasRun::has_failures)
}

struct SdfFallbackApplication {
    report: ScreenSpaceUiTextSdfFallbackReport,
    retained_sdf_run_indices: Vec<usize>,
    native_fallback_run_indices: Vec<usize>,
}

fn apply_sdf_atlas_fallbacks_internal<'a>(
    native_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: &mut Vec<ScreenSpaceUiTextBatch>,
    atlas_runs: &[SdfAtlasRun],
    mut glyph_advances_for_run: impl FnMut(usize) -> Option<&'a [f32]>,
) -> SdfFallbackApplication {
    let pending_sdf_texts = std::mem::take(sdf_texts);
    let mut retained_sdf_texts = Vec::with_capacity(pending_sdf_texts.len());
    let mut retained_sdf_run_indices = Vec::with_capacity(pending_sdf_texts.len());
    let mut native_fallback_run_indices = Vec::new();
    let mut report = ScreenSpaceUiTextSdfFallbackReport::default();

    for (index, text) in pending_sdf_texts.into_iter().enumerate() {
        let Some(run) = atlas_runs.get(index) else {
            report.fallback_text_batch_count = report.fallback_text_batch_count.saturating_add(1);
            report.whole_batch_fallback_text_batch_count = report
                .whole_batch_fallback_text_batch_count
                .saturating_add(1);
            native_texts.push(text);
            native_fallback_run_indices.push(index);
            continue;
        };

        if run.has_failures() {
            let fallback_spans = fallback_spans_for_text_run(text.text.as_str(), run);
            report.record_run_fallback(run, &fallback_spans);

            let glyph_advances = glyph_advances_for_run(index);
            match native_overlay_batches_for_failed_spans(&text, &fallback_spans, glyph_advances) {
                Ok(overlay_batches) => {
                    let overlay_count = overlay_batches.len();
                    report.fallback_native_overlay_batch_count = report
                        .fallback_native_overlay_batch_count
                        .saturating_add(overlay_batches.len());
                    native_texts.extend(overlay_batches);
                    native_fallback_run_indices
                        .extend(std::iter::repeat(index).take(overlay_count));
                    retained_sdf_texts.push(text);
                    retained_sdf_run_indices.push(index);
                }
                Err(reason) => {
                    report.record_mixed_overlay_unsupported(reason);
                    report.whole_batch_fallback_text_batch_count = report
                        .whole_batch_fallback_text_batch_count
                        .saturating_add(1);
                    native_texts.push(text);
                    native_fallback_run_indices.push(index);
                }
            }
        } else {
            retained_sdf_texts.push(text);
            retained_sdf_run_indices.push(index);
        }
    }

    *sdf_texts = retained_sdf_texts;
    SdfFallbackApplication {
        report,
        retained_sdf_run_indices,
        native_fallback_run_indices,
    }
}

impl ScreenSpaceUiTextSdfFallbackReport {
    fn record_run_fallback(
        &mut self,
        run: &SdfAtlasRun,
        fallback_spans: &[SdfAtlasGlyphFallbackSpan],
    ) {
        self.fallback_text_batch_count = self.fallback_text_batch_count.saturating_add(1);
        self.fallback_glyph_count = self
            .fallback_glyph_count
            .saturating_add(run.allocation_failure_count)
            .saturating_add(run.generation_failure_count);
        self.fallback_span_count = self
            .fallback_span_count
            .saturating_add(fallback_spans.len());
        self.fallback_source_byte_count = self
            .fallback_source_byte_count
            .saturating_add(source_byte_count(fallback_spans));
        self.page_limit_glyph_count = self
            .page_limit_glyph_count
            .saturating_add(run.page_limit_failure_count);
        self.oversized_glyph_count = self
            .oversized_glyph_count
            .saturating_add(run.oversized_failure_count);
        for span in fallback_spans {
            self.record_span_fallback(span);
        }
    }

    fn record_span_fallback(&mut self, span: &SdfAtlasGlyphFallbackSpan) {
        let span_source_byte_count = span_source_byte_count(span);
        match span.reason {
            SdfGlyphFallbackReason::Allocation(SdfAtlasAllocationFailureReason::PageLimit) => {
                self.page_limit_span_count = self.page_limit_span_count.saturating_add(1);
                self.page_limit_source_byte_count = self
                    .page_limit_source_byte_count
                    .saturating_add(span_source_byte_count);
            }
            SdfGlyphFallbackReason::Allocation(SdfAtlasAllocationFailureReason::OversizedSlot) => {
                self.oversized_span_count = self.oversized_span_count.saturating_add(1);
                self.oversized_source_byte_count = self
                    .oversized_source_byte_count
                    .saturating_add(span_source_byte_count);
            }
            SdfGlyphFallbackReason::Generation(_) => {}
        }
    }

    fn record_mixed_overlay_unsupported(&mut self, reason: MixedNativeOverlayUnsupportedReason) {
        self.mixed_overlay_unsupported_text_batch_count = self
            .mixed_overlay_unsupported_text_batch_count
            .saturating_add(1);
        match reason {
            MixedNativeOverlayUnsupportedReason::EmptyFallbackSpans => {
                self.mixed_overlay_empty_span_text_batch_count = self
                    .mixed_overlay_empty_span_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::MissingGlyphAdvances => {
                self.mixed_overlay_missing_advances_text_batch_count = self
                    .mixed_overlay_missing_advances_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::UnsupportedWritingMode => {
                self.mixed_overlay_unsupported_writing_mode_text_batch_count = self
                    .mixed_overlay_unsupported_writing_mode_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::UnsupportedTextDirection => {
                self.mixed_overlay_unsupported_text_direction_text_batch_count = self
                    .mixed_overlay_unsupported_text_direction_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::UnsupportedWrap => {
                self.mixed_overlay_unsupported_wrap_text_batch_count = self
                    .mixed_overlay_unsupported_wrap_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::UnsupportedJustify => {
                self.mixed_overlay_unsupported_justify_text_batch_count = self
                    .mixed_overlay_unsupported_justify_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::GlyphAdvanceCountMismatch => {
                self.mixed_overlay_glyph_advance_mismatch_text_batch_count = self
                    .mixed_overlay_glyph_advance_mismatch_text_batch_count
                    .saturating_add(1);
            }
            MixedNativeOverlayUnsupportedReason::InvalidFallbackSpan => {
                self.mixed_overlay_invalid_span_text_batch_count = self
                    .mixed_overlay_invalid_span_text_batch_count
                    .saturating_add(1);
            }
        }
    }
}

fn native_overlay_batches_for_failed_spans(
    text: &ScreenSpaceUiTextBatch,
    fallback_spans: &[SdfAtlasGlyphFallbackSpan],
    glyph_advances: Option<&[f32]>,
) -> Result<Vec<ScreenSpaceUiTextBatch>, MixedNativeOverlayUnsupportedReason> {
    if fallback_spans.is_empty() {
        return Err(MixedNativeOverlayUnsupportedReason::EmptyFallbackSpans);
    }
    let glyph_advances =
        glyph_advances.ok_or(MixedNativeOverlayUnsupportedReason::MissingGlyphAdvances)?;
    let text_direction = validate_mixed_native_overlay_layout_support(text)?;
    let glyph_advances = resolved_layout_advances_for_sdf_glyphs(
        text.text.as_str(),
        glyph_advances,
        text.text.chars().count(),
    )
    .ok_or(MixedNativeOverlayUnsupportedReason::GlyphAdvanceCountMismatch)?;

    fallback_spans
        .iter()
        .map(|span| {
            native_overlay_batch_for_span(text, span, glyph_advances.as_slice(), text_direction)
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(MixedNativeOverlayUnsupportedReason::InvalidFallbackSpan)
}

fn validate_mixed_native_overlay_layout_support(
    text: &ScreenSpaceUiTextBatch,
) -> Result<UiTextDirection, MixedNativeOverlayUnsupportedReason> {
    if !matches!(text.writing_mode, UiTextWritingMode::HorizontalTb) {
        return Err(MixedNativeOverlayUnsupportedReason::UnsupportedWritingMode);
    }
    let text_direction = match text.text_direction {
        UiTextDirection::LeftToRight | UiTextDirection::RightToLeft => text.text_direction,
        UiTextDirection::Auto => {
            resolve_bidi_base_direction(text.text.as_str(), text.text_direction.into()).into()
        }
        UiTextDirection::Mixed => {
            return Err(MixedNativeOverlayUnsupportedReason::UnsupportedTextDirection);
        }
    };
    if !matches!(text.wrap, UiTextWrap::None) {
        return Err(MixedNativeOverlayUnsupportedReason::UnsupportedWrap);
    }
    if matches!(text.text_align, UiTextAlign::Justify) {
        return Err(MixedNativeOverlayUnsupportedReason::UnsupportedJustify);
    }
    Ok(text_direction)
}

fn native_overlay_batch_for_span(
    text: &ScreenSpaceUiTextBatch,
    span: &SdfAtlasGlyphFallbackSpan,
    glyph_advances: &[f32],
    text_direction: UiTextDirection,
) -> Option<ScreenSpaceUiTextBatch> {
    let span_text = text
        .text
        .get(span.start_byte_index..span.end_byte_index)?
        .to_string();
    if span_text.is_empty() {
        return None;
    }

    let full_width = glyph_advances.iter().copied().sum::<f32>();
    let prefix_width = glyph_advances
        .iter()
        .take(span.start_glyph_index)
        .copied()
        .sum::<f32>();
    let span_width = glyph_advances
        .iter()
        .skip(span.start_glyph_index)
        .take(span.glyph_count)
        .copied()
        .sum::<f32>();

    let mut overlay = text.clone();
    overlay.text = span_text;
    overlay.frame = UiFrame::new(
        aligned_text_start_x(text, full_width, text_direction) + prefix_width,
        text.frame.y,
        span_width.max(MIN_NATIVE_OVERLAY_WIDTH),
        text.frame.height,
    );
    overlay.text_align = UiTextAlign::Left;
    overlay.text_direction = text_direction;
    overlay.wrap = UiTextWrap::None;
    Some(overlay)
}

fn aligned_text_start_x(
    text: &ScreenSpaceUiTextBatch,
    text_width: f32,
    text_direction: UiTextDirection,
) -> f32 {
    let free_width = (text.frame.width - text_width).max(0.0);
    let offset = match text.text_align {
        UiTextAlign::Left => 0.0,
        UiTextAlign::Center => free_width * 0.5,
        UiTextAlign::Right => free_width,
        UiTextAlign::Start if matches!(text_direction, UiTextDirection::RightToLeft) => free_width,
        UiTextAlign::Start => 0.0,
        UiTextAlign::End if matches!(text_direction, UiTextDirection::RightToLeft) => 0.0,
        UiTextAlign::End => free_width,
        UiTextAlign::Justify => 0.0,
    };
    text.frame.x + offset
}

// Keep source byte ranges next to glyph spans here so later mixed native/SDF overlay
// code can slice the original string without duplicating run-failure scanning.
fn fallback_spans_for_text_run(
    source_text: &str,
    run: &SdfAtlasRun,
) -> Vec<SdfAtlasGlyphFallbackSpan> {
    let mut spans: Vec<SdfAtlasGlyphFallbackSpan> = Vec::new();
    let mut glyph_index: usize = 0;

    for (byte_index, grapheme) in source_text.grapheme_indices(true) {
        let glyph_count = grapheme.chars().count();
        if glyph_count == 0 {
            continue;
        }
        let reason = (0..glyph_count).find_map(|offset| {
            let glyph_index = glyph_index.saturating_add(offset);
            run.glyph_failure_reasons
                .get(glyph_index)
                .copied()
                .flatten()
                .map(SdfGlyphFallbackReason::Allocation)
                .or_else(|| {
                    run.glyph_generation_failures
                        .get(glyph_index)
                        .copied()
                        .flatten()
                        .map(SdfGlyphFallbackReason::Generation)
                })
        });
        let Some(reason) = reason else {
            glyph_index = glyph_index.saturating_add(glyph_count);
            continue;
        };
        let next_byte_index = byte_index.saturating_add(grapheme.len());

        if let Some(span) = spans.last_mut() {
            let next_index = span.start_glyph_index.saturating_add(span.glyph_count);
            if span.reason == reason && next_index == glyph_index {
                span.glyph_count = span.glyph_count.saturating_add(glyph_count);
                span.end_byte_index = next_byte_index;
                glyph_index = glyph_index.saturating_add(glyph_count);
                continue;
            }
        }

        spans.push(SdfAtlasGlyphFallbackSpan {
            start_glyph_index: glyph_index,
            glyph_count,
            start_byte_index: byte_index,
            end_byte_index: next_byte_index,
            reason,
        });
        glyph_index = glyph_index.saturating_add(glyph_count);
    }

    spans
}

fn source_byte_count(spans: &[SdfAtlasGlyphFallbackSpan]) -> usize {
    spans.iter().map(span_source_byte_count).sum()
}

fn span_source_byte_count(span: &SdfAtlasGlyphFallbackSpan) -> usize {
    span.end_byte_index.saturating_sub(span.start_byte_index)
}

#[cfg(test)]
mod tests;
