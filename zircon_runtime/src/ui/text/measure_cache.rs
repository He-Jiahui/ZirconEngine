use crate::core::framework::text::TextDirection;
use crate::core::runtime::tasks::TaskPool;
use crate::text::{
    SharedTextLayoutSession, TextDocumentKey, TextRange,
    cache::{
        DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY, DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
        ShapedRunCacheReport, TextFrameDedup, TextFrameDedupReport, TextLayoutCache,
        TextLayoutCacheReport, TextLayoutWidthValidity, TextMeasureCache, TextMeasureCacheReport,
    },
    font::shared_font_database_generation,
    layout::measure_line_width,
    parallel::shape_pool::{TextParallelShapeBatchReport, TextShapeParagraph},
    text_style,
};
use std::{
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextDirection, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::resolved_layout::{
    UiTextLayoutRequest, UiTextLayoutResolution, UiTextStyleKey, resolve_text_layout_with_provider,
    resolve_text_layout_with_provider_and_parsed,
};
use super::rich_text::{UiParsedText, parse_source_text};
use super::shaper::{
    measure_text_size_with_provider as measure_backend_text_size_with_provider,
    measure_unwrapped_text_height_with_provider,
};

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) struct UiWidthBucket(u32);

impl UiWidthBucket {
    pub(crate) fn from_request(request: &UiTextLayoutRequest<'_>) -> Self {
        if request.style.wrap == UiTextWrap::None {
            return Self(0);
        }

        let advance = measure_line_width("n", &text_style(request.style))
            .max(request.style.font_size.max(1.0) * 0.25)
            .max(1.0);
        Self(
            (request.frame.width.max(advance) / advance)
                .floor()
                .max(1.0) as u32,
        )
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureKey {
    pub content_hash: u64,
    pub frame: UiFrameKey,
    pub clip_frame: Option<UiFrameKey>,
    pub viewport: Option<(u32, u32, usize)>,
    pub width_bucket: UiWidthBucket,
    pub style: UiTextStyleKey,
    pub font_database_generation: u64,
}

impl Hash for UiTextMeasureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.frame.hash(state);
        self.clip_frame.hash(state);
        self.viewport.hash(state);
        self.width_bucket.hash(state);
        self.style.hash(state);
        self.font_database_generation.hash(state);
    }
}

impl UiTextMeasureKey {
    pub(crate) fn from_request(request: &UiTextLayoutRequest<'_>) -> Self {
        Self::from_request_at_generation(request, shared_font_database_generation())
    }

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
            width_bucket: UiWidthBucket::from_request(request),
            style: request.style_key(),
            font_database_generation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureSizeKey {
    pub content_hash: u64,
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
    pub(crate) fn from_text_style(
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> Self {
        Self::from_text_style_at_generation(text, style, shared_font_database_generation())
    }

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
}

#[cfg(test)]
mod generation_key_tests {
    use std::sync::Arc;

    use super::{UiTextMeasureCache, UiTextMeasureKey, UiTextMeasureSizeKey};
    use crate::text::TextDocumentKey;
    use crate::ui::text::{UiTextLayoutRequest, UiTextViewport};
    use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiResolvedStyle};

    #[test]
    fn ui_text_cache_keys_change_with_font_database_generation() {
        let style = UiResolvedStyle::default();
        let request = UiTextLayoutRequest::new(
            "generation",
            &style,
            UiFrame::new(0.0, 0.0, 100.0, 20.0),
            None,
        );

        assert_ne!(
            UiTextMeasureSizeKey::from_text_style_at_generation("generation", &style, 1),
            UiTextMeasureSizeKey::from_text_style_at_generation("generation", &style, 2)
        );
        assert_ne!(
            UiTextMeasureKey::from_request_at_generation(&request, 1),
            UiTextMeasureKey::from_request_at_generation(&request, 2)
        );
    }

    #[test]
    fn retained_plain_document_cache_reuses_a_document_revision() {
        let style = UiResolvedStyle::default();
        let frame = UiFrame::new(0.0, 0.0, 120.0, 40.0);
        let viewport = UiTextViewport::new(0.0, 40.0, 0).expect("finite viewport");
        let mut cache = UiTextMeasureCache::default();
        let first_request = UiTextLayoutRequest::new("first\nsecond", &style, frame, None)
            .with_viewport(viewport)
            .with_document_key(TextDocumentKey::new(7, 1));
        let repeat_request = UiTextLayoutRequest::new("first\nsecond", &style, frame, None)
            .with_viewport(viewport)
            .with_document_key(TextDocumentKey::new(7, 1));
        let revised_request = UiTextLayoutRequest::new("first\nsecond\nthird", &style, frame, None)
            .with_viewport(viewport)
            .with_document_key(TextDocumentKey::new(7, 2));

        let first = cache.retained_plain_document(&first_request);
        let repeated = cache.retained_plain_document(&repeat_request);
        let revised = cache.retained_plain_document(&revised_request);

        assert!(Arc::ptr_eq(&first.rich, &repeated.rich));
        assert!(!Arc::ptr_eq(&first.rich, &revised.rich));
        assert_eq!(cache.retained_plain_documents.len(), 2);
        assert!(cache.retained_plain_document_bytes <= RETAINED_PLAIN_DOCUMENT_MAX_BYTES);
    }
}

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

fn text_hash(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextShapePrewarmRequest {
    text: Arc<str>,
    style: UiResolvedStyle,
}

const RETAINED_PLAIN_DOCUMENT_CAPACITY: usize = 16;
const RETAINED_PLAIN_DOCUMENT_MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
struct RetainedPlainTextDocument {
    key: TextDocumentKey,
    parsed: UiParsedText,
    estimated_bytes: usize,
    last_access: u64,
}

impl UiTextShapePrewarmRequest {
    pub(crate) fn horizontal(text: impl Into<Arc<str>>, style: UiResolvedStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub(crate) fn from_layout_source(text: &str, style: UiResolvedStyle) -> Option<Self> {
        if text.is_empty()
            || !matches!(style.rich_text_format, UiRichTextFormat::Plain)
            || !matches!(style.text_writing_mode, UiTextWritingMode::HorizontalTb)
        {
            return None;
        }
        Some(Self {
            text: Arc::from(text),
            style,
        })
    }

    fn to_shape_paragraphs(&self) -> Vec<TextShapeParagraph> {
        TextShapeParagraph::horizontal_paragraphs(
            self.text.as_ref(),
            text_style(&self.style),
            TextDirection::Auto,
            TextRange {
                start: 0,
                end: self.text.len(),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextMeasureCache {
    measure_frame_dedup: TextFrameDedup<UiTextMeasureSizeKey, UiSize>,
    measure_cache: TextMeasureCache<UiTextMeasureSizeKey, UiSize>,
    text_layout_session: SharedTextLayoutSession,
    layout_frame_dedup: TextFrameDedup<UiTextMeasureKey, UiTextLayoutResolution>,
    layout_cache: TextLayoutCache<UiTextMeasureKey, UiTextLayoutResolution>,
    retained_plain_documents: Vec<RetainedPlainTextDocument>,
    retained_plain_document_bytes: usize,
    retained_plain_document_access: u64,
    shape_prewarm_report: TextParallelShapeBatchReport,
    frame_index: u64,
}

impl Default for UiTextMeasureCache {
    fn default() -> Self {
        Self {
            measure_frame_dedup: TextFrameDedup::default(),
            measure_cache: TextMeasureCache::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY),
            text_layout_session: SharedTextLayoutSession::new(),
            layout_frame_dedup: TextFrameDedup::default(),
            layout_cache: TextLayoutCache::with_capacity(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY),
            retained_plain_documents: Vec::new(),
            retained_plain_document_bytes: 0,
            retained_plain_document_access: 0,
            shape_prewarm_report: TextParallelShapeBatchReport::default(),
            frame_index: 0,
        }
    }
}

impl UiTextMeasureCache {
    pub(crate) fn clear(&mut self) {
        self.measure_frame_dedup.clear();
        self.measure_cache.clear();
        self.text_layout_session.clear();
        self.layout_frame_dedup.clear();
        self.layout_cache.clear();
        self.retained_plain_documents.clear();
        self.retained_plain_document_bytes = 0;
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame_index = self.frame_index.saturating_add(1);
        self.measure_frame_dedup.begin_frame(self.frame_index);
        self.measure_cache.begin_frame(self.frame_index);
        self.text_layout_session.begin_frame(self.frame_index);
        self.layout_frame_dedup.begin_frame(self.frame_index);
        self.layout_cache.begin_frame(self.frame_index);
        self.shape_prewarm_report = TextParallelShapeBatchReport::default();
    }

    pub(crate) fn finish_frame(&mut self) {
        self.measure_cache.finish_frame();
        self.text_layout_session.finish_frame();
        self.layout_cache.finish_frame();
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

    pub(crate) fn frame_shape_prewarm_report(&self) -> TextParallelShapeBatchReport {
        self.shape_prewarm_report
    }

    pub(crate) fn frame_layout_report(&self) -> TextLayoutCacheReport {
        self.layout_cache.report()
    }

    pub(crate) fn frame_layout_dedup_report(&self) -> TextFrameDedupReport {
        self.layout_frame_dedup.report()
    }

    pub(crate) fn prewarm_horizontal_paragraphs(
        &mut self,
        pool: &TaskPool,
        requests: &[UiTextShapePrewarmRequest],
        chunk_size: usize,
    ) -> TextParallelShapeBatchReport {
        let paragraphs = requests
            .iter()
            .flat_map(UiTextShapePrewarmRequest::to_shape_paragraphs)
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
        self.shape_prewarm_report.generation_deferred_count = self
            .shape_prewarm_report
            .generation_deferred_count
            .saturating_add(report.generation_deferred_count);
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
    }

    pub(crate) fn measure_text_size(
        &mut self,
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> UiSize {
        if text.is_empty() {
            return UiSize::default();
        }

        let key = UiTextMeasureSizeKey::from_text_style(text, style);
        if let Some(size) = self.measure_frame_dedup.get(&key, text).copied() {
            return size;
        }

        let size = if let Some(size) = self.measure_cache.get(&key, text).copied() {
            size
        } else {
            let measured =
                measure_backend_text_size_with_provider(text, style, &mut self.text_layout_session);
            *self.measure_cache.insert(key.clone(), text, measured)
        };
        self.measure_frame_dedup.insert(key, text, size);
        size
    }

    pub(crate) fn measure_unwrapped_text_height(
        &mut self,
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> Option<f32> {
        measure_unwrapped_text_height_with_provider(text, style, &mut self.text_layout_session)
    }

    pub(crate) fn resolve_or_shape(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> UiTextLayoutResolution {
        if request.has_stable_viewport_document()
            && matches!(request.style.rich_text_format, UiRichTextFormat::Plain)
        {
            // Virtualized retained documents provide a revisioned identity. Avoid copying or
            // scanning the full source just to cache each distinct scroll window.
            let parsed = self.retained_plain_document(request);
            return resolve_text_layout_with_provider_and_parsed(
                request,
                &parsed,
                &mut self.text_layout_session,
            );
        }
        let key = UiTextMeasureKey::from_request(request);
        let resolved_text = request.resolved_text();
        if let Some(resolution) = self
            .layout_frame_dedup
            .get(&key, resolved_text.as_ref())
            .cloned()
        {
            return resolution;
        }

        let resolved_text: Arc<str> = Arc::from(resolved_text.as_ref());
        let width_validity = TextLayoutWidthValidity::exact(request.frame.width);
        let resolution = if let Some(resolution) = self
            .layout_cache
            .get(&key, resolved_text.as_ref(), request.frame.width)
            .cloned()
        {
            resolution
        } else {
            let resolution =
                resolve_text_layout_with_provider(request, &mut self.text_layout_session);
            self.layout_cache
                .insert(
                    key.clone(),
                    Arc::clone(&resolved_text),
                    width_validity,
                    resolution,
                )
                .clone()
        };
        self.layout_frame_dedup
            .insert(key, resolved_text, resolution.clone());
        resolution
    }

    fn retained_plain_document(&mut self, request: &UiTextLayoutRequest<'_>) -> UiParsedText {
        let Some(key) = request.document_key else {
            return parse_source_text(request.text, crate::text::RichTextFormat::Plain);
        };
        self.retained_plain_document_access = self.retained_plain_document_access.saturating_add(1);
        let access = self.retained_plain_document_access;
        if let Some(index) = self
            .retained_plain_documents
            .iter()
            .position(|document| document.key == key)
        {
            let document = &mut self.retained_plain_documents[index];
            document.last_access = access;
            return document.parsed.clone();
        }

        let parsed = parse_source_text(request.text, crate::text::RichTextFormat::Plain);
        let estimated_bytes = parsed
            .estimated_bytes()
            .saturating_add(size_of::<RetainedPlainTextDocument>());
        if estimated_bytes > RETAINED_PLAIN_DOCUMENT_MAX_BYTES {
            return parsed;
        }
        while self.retained_plain_documents.len() >= RETAINED_PLAIN_DOCUMENT_CAPACITY
            || self
                .retained_plain_document_bytes
                .saturating_add(estimated_bytes)
                > RETAINED_PLAIN_DOCUMENT_MAX_BYTES
        {
            if let Some((oldest_index, _)) = self
                .retained_plain_documents
                .iter()
                .enumerate()
                .min_by_key(|(_, document)| document.last_access)
            {
                let removed = self.retained_plain_documents.swap_remove(oldest_index);
                self.retained_plain_document_bytes = self
                    .retained_plain_document_bytes
                    .saturating_sub(removed.estimated_bytes);
            } else {
                break;
            }
        }
        self.retained_plain_document_bytes = self
            .retained_plain_document_bytes
            .saturating_add(estimated_bytes);
        self.retained_plain_documents
            .push(RetainedPlainTextDocument {
                key,
                parsed: parsed.clone(),
                estimated_bytes,
                last_access: access,
            });
        parsed
    }
}
