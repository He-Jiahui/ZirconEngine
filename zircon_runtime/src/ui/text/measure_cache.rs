use crate::core::framework::text::TextDirection;
use crate::core::runtime::tasks::TaskPool;
#[cfg(feature = "profiling")]
use crate::text::CompiledRichTextCacheReport;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{
    EphemeralCacheHash, RichSemanticProjection, RichTextFormat, SharedTextLayoutSession,
    TextDocumentKey, TextRange, TextStyle, VerticalMode,
    cache::{
        DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY, DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
        HardLineIndexCacheReport, ShapedRunCacheReport, TextFrameDedup, TextFrameDedupReport,
        TextLayoutCache, TextLayoutCacheReport, TextLayoutWidthValidity, TextMeasureCache,
        TextMeasureCacheReport,
    },
    font::{FontCollectionService, shared_font_collection_service},
    from_compiled_rich_semantic_projection, has_multiple_hard_lines,
    layout::resolved_text_spans,
    parallel::shape_pool::{TextParallelShapeBatchReport, TextShapeParagraph},
    text_style,
};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedStyle, UiRichTextFormat, UiTextDirection, UiTextOverflow,
    UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::layout_engine::viewport_selects_partial_plain_text as layout_viewport_selects_partial_plain_text;
use super::layout_engine::{
    measure_text_size_with_provider_outcome, resolve_text_direction, text_layout_error_layout,
};
use super::resolved_layout::{
    UiTextLayoutRequest, UiTextLayoutResolution, UiTextStyleKey, resolution_from_layout,
    resolve_text_layout_with_provider_and_parsed_outcome,
    resolve_text_layout_with_provider_outcome,
};
use super::rich_text::{UiParsedText, parse_source_text_with_provider};
use super::shaper::measure_unwrapped_text_height_with_provider;

mod retained_document;

#[cfg(test)]
use retained_document::RETAINED_PLAIN_DOCUMENT_MAX_BYTES;
use retained_document::RetainedPlainTextDocumentCache;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureKey {
    pub content_hash: EphemeralCacheHash,
    pub frame: UiFrameKey,
    pub clip_frame: Option<UiFrameKey>,
    pub viewport: Option<(u32, u32, usize)>,
    pub style: UiTextStyleKey,
    pub font_database_generation: u64,
}

impl Hash for UiTextMeasureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.frame.hash(state);
        self.clip_frame.hash(state);
        self.viewport.hash(state);
        self.style.hash(state);
        self.font_database_generation.hash(state);
    }
}

impl UiTextMeasureKey {
    fn from_request_at_generation(
        request: &UiTextLayoutRequest<'_>,
        font_database_generation: u64,
    ) -> Self {
        Self {
            content_hash: request.source_hash(),
            frame: UiFrameKey::from_frame(request.frame),
            clip_frame: request.clip_frame.map(UiFrameKey::from_frame),
            viewport: request
                .layout_viewport()
                .map(|viewport| viewport.cache_key()),
            style: request.style_key(),
            font_database_generation,
        }
    }

    fn estimated_heap_bytes(&self) -> usize {
        self.style.estimated_heap_bytes()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureSizeKey {
    pub content_hash: EphemeralCacheHash,
    pub style: UiTextStyleKey,
    pub font_database_generation: u64,
}

impl Hash for UiTextMeasureSizeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.style.hash(state);
        self.font_database_generation.hash(state);
    }
}

impl UiTextMeasureSizeKey {
    fn from_text_style_at_generation(
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
        font_database_generation: u64,
    ) -> Self {
        Self {
            content_hash: text_hash(text),
            style: UiTextStyleKey::from_style(style),
            font_database_generation,
        }
    }

    fn estimated_heap_bytes(&self) -> usize {
        self.style.estimated_heap_bytes()
    }
}

#[cfg(test)]
mod generation_key_tests;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct UiFrameKey {
    x_bits: u32,
    y_bits: u32,
    width_bits: u32,
    height_bits: u32,
}

impl UiFrameKey {
    fn from_frame(frame: UiFrame) -> Self {
        Self {
            x_bits: normalized_bits(frame.x),
            y_bits: normalized_bits(frame.y),
            width_bits: normalized_bits(frame.width),
            height_bits: normalized_bits(frame.height),
        }
    }
}

fn normalized_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

fn text_hash(text: &str) -> EphemeralCacheHash {
    EphemeralCacheHash::from_hashable(text)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextShapePrewarmRequest {
    paragraphs: Vec<TextShapeParagraph>,
}

impl UiTextShapePrewarmRequest {
    pub(crate) fn horizontal(text: impl Into<Arc<str>>, style: UiResolvedStyle) -> Self {
        let text = text.into();
        Self {
            paragraphs: TextShapeParagraph::horizontal_paragraphs(
                text.as_ref(),
                text_style(&style),
                TextDirection::Auto,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
        }
    }

    fn from_layout_source(
        text: &str,
        style: UiResolvedStyle,
        provider: &SharedTextLayoutSession,
    ) -> Option<Self> {
        if text.is_empty() {
            return None;
        }
        if matches!(style.rich_text_format, UiRichTextFormat::Plain)
            && matches!(style.text_writing_mode, UiTextWritingMode::HorizontalTb)
        {
            return Some(Self::horizontal(text, style));
        }

        let vertical_mode = matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
            .then_some(VerticalMode::Mixed);
        let base_style = text_style(&style);
        let parsed =
            parse_source_text_with_provider(text, style.rich_text_format.into(), provider).ok()?;
        let paragraphs: Vec<TextShapeParagraph> =
            if parsed.runs.iter().any(|run| run.inline().is_some()) {
                // Inline rich layout routes through RichAdvanceIndex. Reuse its exact resolved-span
                // projection so adjacent runs with the same effective style share the cache key.
                resolved_text_spans(&parsed, &base_style)
                    .ok()?
                    .into_iter()
                    .flat_map(|span| {
                        parsed
                            .text()
                            .get(span.start..span.end)
                            .into_iter()
                            .flat_map(move |text| {
                                rich_layout_span_shape_paragraphs(
                                    text,
                                    span.style.clone(),
                                    vertical_mode,
                                )
                            })
                    })
                    .collect()
            } else {
                // Non-inline layout resolves canonical candidate-line metrics after source runs are
                // assembled. Prewarm its complete hard line so the cache key serves that line without
                // speculatively materializing competing per-run variants before wrapping is known.
                layout_hard_line_shape_paragraphs(parsed.text(), base_style, vertical_mode)
            };
        (!paragraphs.is_empty()).then_some(Self { paragraphs })
    }
}

fn rich_layout_span_shape_paragraphs(
    text: &str,
    style: TextStyle,
    vertical_mode: Option<VerticalMode>,
) -> Vec<TextShapeParagraph> {
    layout_hard_line_shape_paragraphs(text, style, vertical_mode)
}

fn layout_hard_line_shape_paragraphs(
    text: &str,
    style: TextStyle,
    vertical_mode: Option<VerticalMode>,
) -> Vec<TextShapeParagraph> {
    crate::text::hard_lines(text)
        .into_iter()
        .filter_map(|line| {
            text.get(line.content)
                .filter(|content| !content.is_empty())
                .map(|content| {
                    TextShapeParagraph::layout_span(
                        content,
                        style.clone(),
                        TextDirection::Auto,
                        vertical_mode,
                    )
                })
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextMeasureCache {
    measure_frame_dedup: TextFrameDedup<UiTextMeasureSizeKey, UiSize>,
    measure_cache: TextMeasureCache<UiTextMeasureSizeKey, UiSize>,
    text_layout_session: SharedTextLayoutSession,
    layout_frame_dedup: TextFrameDedup<UiTextMeasureKey, UiTextLayoutResolution>,
    layout_cache: TextLayoutCache<UiTextMeasureKey, UiTextLayoutResolution>,
    retained_plain_documents: RetainedPlainTextDocumentCache,
    uncached_document_resolve_count: usize,
    shape_prewarm_report: TextParallelShapeBatchReport,
    frame_index: u64,
}

impl Default for UiTextMeasureCache {
    /// Creates a standalone cache backed by the process-owner font collection.
    /// Retained surfaces construct this cache with their selected owner collection: the Editor
    /// process collection or a Runtime Core-owned collection.
    fn default() -> Self {
        Self::new_with_font_collection(shared_font_collection_service())
    }
}

impl UiTextMeasureCache {
    pub(crate) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        let text_layout_session =
            SharedTextLayoutSession::new_with_font_collection(font_collection);
        Self {
            measure_frame_dedup: TextFrameDedup::default(),
            measure_cache: TextMeasureCache::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY),
            text_layout_session,
            layout_frame_dedup: TextFrameDedup::default(),
            layout_cache: TextLayoutCache::with_capacity(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY),
            retained_plain_documents: RetainedPlainTextDocumentCache::default(),
            uncached_document_resolve_count: 0,
            shape_prewarm_report: TextParallelShapeBatchReport::default(),
            frame_index: 0,
        }
    }

    pub(crate) fn font_database_generation(&self) -> u64 {
        self.text_layout_session.font_database_generation()
    }

    pub(crate) fn shape_prewarm_request(
        &self,
        text: &str,
        style: UiResolvedStyle,
    ) -> Option<UiTextShapePrewarmRequest> {
        UiTextShapePrewarmRequest::from_layout_source(text, style, &self.text_layout_session)
    }

    pub(crate) fn compile_rich_semantic_projection(
        &self,
        source_markup: &str,
        format: RichTextFormat,
    ) -> Option<RichSemanticProjection> {
        let compiled = self
            .text_layout_session
            .compile_rich_text(source_markup, format)
            .ok()?;
        from_compiled_rich_semantic_projection(compiled, source_markup, format)
    }

    pub(crate) fn font_collection_snapshot(&self) -> crate::text::font::FontCollectionSnapshot {
        self.text_layout_session.font_collection_snapshot()
    }

    pub(crate) fn clear(&mut self) {
        self.measure_frame_dedup.clear();
        self.measure_cache.clear();
        self.text_layout_session.clear();
        self.layout_frame_dedup.clear();
        self.layout_cache.clear();
        self.retained_plain_documents.clear();
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame_index = self.frame_index.saturating_add(1);
        self.measure_frame_dedup.begin_frame(self.frame_index);
        self.measure_cache.begin_frame(self.frame_index);
        self.text_layout_session.begin_frame(self.frame_index);
        self.layout_frame_dedup.begin_frame(self.frame_index);
        self.layout_cache.begin_frame(self.frame_index);
        self.uncached_document_resolve_count = 0;
        self.shape_prewarm_report = TextParallelShapeBatchReport::default();
    }

    pub(crate) fn finish_frame(&mut self) {
        self.measure_cache.finish_frame();
        self.text_layout_session.finish_frame();
        self.layout_cache.finish_frame();
    }

    pub(crate) fn prepare_render_command_text_artifacts(
        &mut self,
        commands: &mut [UiRenderCommand],
    ) {
        super::rich_text::prepare_render_command_text_artifacts_with_provider(
            commands,
            &mut self.text_layout_session,
        );
    }

    pub(crate) fn frame_shape_count(&self) -> u64 {
        self.layout_cache.report().miss_count
    }

    pub(crate) fn frame_measure_report(&self) -> TextMeasureCacheReport {
        self.measure_cache.report()
    }

    pub(crate) fn frame_measure_dedup_report(&self) -> TextFrameDedupReport {
        self.measure_frame_dedup.report()
    }

    pub(crate) fn frame_shaped_run_report(&self) -> ShapedRunCacheReport {
        self.text_layout_session.cache_report()
    }

    pub(crate) fn hard_line_index_report(&self) -> HardLineIndexCacheReport {
        self.text_layout_session.hard_line_index_report()
    }

    pub(crate) const fn frame_shaping_work_report(&self) -> crate::text::TextShapingWorkReport {
        self.text_layout_session.shaping_work_report()
    }

    pub(crate) const fn frame_layout_session_diagnostics(
        &self,
    ) -> crate::text::TextLayoutSessionDiagnostics {
        self.text_layout_session.diagnostics_report()
    }

    pub(crate) fn frame_shape_prewarm_report(&self) -> TextParallelShapeBatchReport {
        self.shape_prewarm_report
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn sample_compiled_rich_text_cache(&mut self) -> CompiledRichTextCacheReport {
        self.text_layout_session
            .take_compiled_rich_text_cache_report()
    }

    pub(crate) fn frame_layout_report(&self) -> TextLayoutCacheReport {
        self.layout_cache.report()
    }

    pub(crate) fn frame_layout_dedup_report(&self) -> TextFrameDedupReport {
        self.layout_frame_dedup.report()
    }

    pub(crate) const fn frame_uncached_document_resolve_count(&self) -> usize {
        self.uncached_document_resolve_count
    }

    pub(crate) fn prewarm_horizontal_paragraphs(
        &mut self,
        pool: &TaskPool,
        requests: &[UiTextShapePrewarmRequest],
        chunk_size: usize,
    ) -> TextParallelShapeBatchReport {
        let paragraphs = requests
            .iter()
            .flat_map(|request| request.paragraphs.iter().cloned())
            .collect::<Vec<_>>();
        let report =
            self.text_layout_session
                .prewarm_horizontal_paragraphs(pool, &paragraphs, chunk_size);
        self.record_shape_prewarm_report(report);
        report
    }

    fn record_shape_prewarm_report(&mut self, report: TextParallelShapeBatchReport) {
        self.shape_prewarm_report.requested_count = self
            .shape_prewarm_report
            .requested_count
            .saturating_add(report.requested_count);
        self.shape_prewarm_report.cache_hit_count = self
            .shape_prewarm_report
            .cache_hit_count
            .saturating_add(report.cache_hit_count);
        self.shape_prewarm_report.cache_miss_count = self
            .shape_prewarm_report
            .cache_miss_count
            .saturating_add(report.cache_miss_count);
        self.shape_prewarm_report.batch_duplicate_count = self
            .shape_prewarm_report
            .batch_duplicate_count
            .saturating_add(report.batch_duplicate_count);
        self.shape_prewarm_report.pending_lookup_candidate_count = self
            .shape_prewarm_report
            .pending_lookup_candidate_count
            .saturating_add(report.pending_lookup_candidate_count);
        self.shape_prewarm_report.shaped_count = self
            .shape_prewarm_report
            .shaped_count
            .saturating_add(report.shaped_count);
        self.shape_prewarm_report.inserted_count = self
            .shape_prewarm_report
            .inserted_count
            .saturating_add(report.inserted_count);
        self.shape_prewarm_report.invalid_request_count = self
            .shape_prewarm_report
            .invalid_request_count
            .saturating_add(report.invalid_request_count);
        self.shape_prewarm_report.generation_deferred_count = self
            .shape_prewarm_report
            .generation_deferred_count
            .saturating_add(report.generation_deferred_count);
        self.shape_prewarm_report.failed_count = self
            .shape_prewarm_report
            .failed_count
            .saturating_add(report.failed_count);
        self.shape_prewarm_report.inline_batch_count = self
            .shape_prewarm_report
            .inline_batch_count
            .saturating_add(report.inline_batch_count);
        self.shape_prewarm_report.parallel_join_count = self
            .shape_prewarm_report
            .parallel_join_count
            .saturating_add(report.parallel_join_count);
        self.shape_prewarm_report.caller_wait_nanos = self
            .shape_prewarm_report
            .caller_wait_nanos
            .saturating_add(report.caller_wait_nanos);
        self.shape_prewarm_report.chunk_size =
            self.shape_prewarm_report.chunk_size.max(report.chunk_size);
        self.shape_prewarm_report.worker_parallelism = self
            .shape_prewarm_report
            .worker_parallelism
            .max(report.worker_parallelism);
        self.shape_prewarm_report
            .shaping_work
            .merge(report.shaping_work);
    }

    pub(crate) fn measure_text_size(
        &mut self,
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> UiSize {
        if text.is_empty() {
            return UiSize::default();
        }

        let key = UiTextMeasureSizeKey::from_text_style_at_generation(
            text,
            style,
            self.font_database_generation(),
        );
        if let Some(size) = self.measure_frame_dedup.get(&key, text).copied() {
            return size;
        }

        let (stored_text, size) = if let Some((stored_text, size)) =
            self.measure_cache.get_with_stored_text(&key, text)
        {
            (Arc::clone(stored_text), *size)
        } else {
            let measured = match measure_text_size_with_provider_outcome(
                text,
                style,
                &mut self.text_layout_session,
            ) {
                TextShapingOutcome::Ready(size) => size,
                TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                    self.text_layout_session.record_layout_error(&error);
                    return UiSize::default();
                }
            };
            let stored_text: Arc<str> = Arc::from(text);
            let key_heap_bytes = key.estimated_heap_bytes();
            let size = *self.measure_cache.insert_with_additional_heap_bytes(
                key.clone(),
                Arc::clone(&stored_text),
                measured,
                key_heap_bytes,
            );
            (stored_text, size)
        };
        self.measure_frame_dedup.insert(key, stored_text, size);
        size
    }

    pub(crate) fn measure_unwrapped_text_height(
        &mut self,
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> Option<f32> {
        measure_unwrapped_text_height_with_provider(text, style, &mut self.text_layout_session)
    }

    /// Returns true only when layout will materialize a strict hard-line subset for this owner.
    /// This preserves full-document prewarm for short source even when its clip is smaller than
    /// its frame, while allowing retained logs to avoid shaping every paragraph first.
    pub(crate) fn viewport_selects_partial_plain_text(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> bool {
        self.retained_plain_document_for_viewport(request)
            .is_some_and(|(_, is_partial)| is_partial)
    }

    pub(crate) fn resolve_or_shape(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> UiTextLayoutResolution {
        match self.resolve_or_shape_outcome(request) {
            TextShapingOutcome::Ready(resolution) => resolution,
            TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                self.safe_layout_resolution(request, &error)
            }
        }
    }

    /// A non-ready layout never enters frame or persistent caches.
    pub(crate) fn resolve_or_shape_outcome(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> TextLayoutOutcome<UiTextLayoutResolution> {
        let key =
            UiTextMeasureKey::from_request_at_generation(request, self.font_database_generation());
        let resolved_text = request.resolved_text();
        if let Some(resolution) = self
            .layout_frame_dedup
            .get(&key, resolved_text.as_ref())
            .cloned()
        {
            return TextShapingOutcome::Ready(resolution);
        }

        let width_validity = TextLayoutWidthValidity::exact(request.frame.width);
        if let Some((stored_text, resolution)) = self.layout_cache.get_with_stored_text(
            &key,
            resolved_text.as_ref(),
            request.frame.width,
        ) {
            let resolution = resolution.clone();
            self.layout_frame_dedup
                .insert(key, Arc::clone(stored_text), resolution.clone());
            return TextShapingOutcome::Ready(resolution);
        }

        let complete_viewport_document = match self.retained_plain_document_for_viewport(request) {
            Some((parsed, true)) => {
                // A strict hard-line subset has no reusable complete-document layout. Keep its
                // parsed document and hard-line index, but do not let viewport-specific geometry
                // enter the persistent cache.
                self.uncached_document_resolve_count =
                    self.uncached_document_resolve_count.saturating_add(1);
                let resolution = match resolve_text_layout_with_provider_and_parsed_outcome(
                    request,
                    &parsed,
                    &mut self.text_layout_session,
                ) {
                    TextShapingOutcome::Ready(resolution) => resolution,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                self.layout_frame_dedup
                    .insert(key, parsed.rich.shared_text(), resolution.clone());
                return TextShapingOutcome::Ready(resolution);
            }
            Some((parsed, false)) => Some(parsed),
            None => None,
        };

        let resolution = match match complete_viewport_document {
            Some(parsed) => resolve_text_layout_with_provider_and_parsed_outcome(
                request,
                &parsed,
                &mut self.text_layout_session,
            ),
            None => {
                resolve_text_layout_with_provider_outcome(request, &mut self.text_layout_session)
            }
        } {
            TextShapingOutcome::Ready(resolution) => resolution,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        let resolved_text: Arc<str> = Arc::from(resolved_text.as_ref());
        let additional_heap_bytes = key
            .estimated_heap_bytes()
            .saturating_add(resolution.estimated_cache_heap_bytes());
        let resolution = self
            .layout_cache
            .insert_with_additional_heap_bytes(
                key.clone(),
                Arc::clone(&resolved_text),
                width_validity,
                resolution,
                additional_heap_bytes,
            )
            .clone();
        self.layout_frame_dedup
            .insert(key, resolved_text, resolution.clone());
        TextShapingOutcome::Ready(resolution)
    }

    fn safe_layout_resolution(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
        error: &crate::core::framework::text::TextLayoutError,
    ) -> UiTextLayoutResolution {
        let layout = text_layout_error_layout(
            request.style,
            resolve_text_direction(request.text, request.style.text_direction),
            request.style.font_size.max(1.0),
            request
                .style
                .line_height
                .max(request.style.font_size)
                .max(1.0),
            request.text.len(),
            error,
            &mut self.text_layout_session,
        );
        resolution_from_layout(request, layout)
    }

    fn retained_plain_document_for_viewport(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> Option<(UiParsedText, bool)> {
        let viewport = request.layout_viewport()?;
        if !request.supports_viewport_virtualized_plain_layout() {
            return None;
        }
        if !has_multiple_hard_lines(request.text) {
            return None;
        }

        let parsed = self.retained_plain_document(request).ok()?;
        let is_partial = layout_viewport_selects_partial_plain_text(
            &parsed,
            request.style,
            viewport,
            request.document_key,
            &mut self.text_layout_session,
        );
        Some((parsed, is_partial))
    }

    fn retained_plain_document(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> Result<UiParsedText, crate::core::framework::text::TextLayoutError> {
        let Some(key) = request.document_key else {
            return parse_source_text_with_provider(
                request.text,
                crate::text::RichTextFormat::Plain,
                &self.text_layout_session,
            );
        };
        self.retained_plain_documents
            .resolve(key, request.text, &self.text_layout_session)
    }
}
