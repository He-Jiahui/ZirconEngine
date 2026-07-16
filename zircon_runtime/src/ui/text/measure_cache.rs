use crate::core::framework::text::TextDirection;
use crate::core::runtime::tasks::TaskPool;
use crate::text::{
    cache::{
        ShapedRunCacheReport, TextFrameDedup, TextFrameDedupReport, TextLayoutCache,
        TextLayoutCacheReport, TextLayoutWidthValidity, TextMeasureCache, TextMeasureCacheReport,
        DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY, DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
    },
    layout::measure_line_width,
    parallel::shape_pool::{TextParallelShapeBatchReport, TextShapeParagraph},
    SharedTextLayoutSession, TextRange,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextDirection, UiTextRange, UiTextWrap,
};

use super::adapter::text_style;
use super::resolved_layout::{
    resolve_text_layout_with_provider, UiTextLayoutRequest, UiTextLayoutResolution, UiTextStyleKey,
};
use super::rich_text::parse_source_text;
use super::shaper::measure_text_size_with_provider as measure_backend_text_size_with_provider;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    pub width_bucket: UiWidthBucket,
    pub style: UiTextStyleKey,
}

impl UiTextMeasureKey {
    pub(crate) fn from_request(request: &UiTextLayoutRequest<'_>) -> Self {
        Self {
            content_hash: request.source_hash(),
            frame: UiFrameKey::from_frame(request.frame),
            clip_frame: request.clip_frame.map(UiFrameKey::from_frame),
            width_bucket: UiWidthBucket::from_request(request),
            style: request.style_key(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureSizeKey {
    pub content_hash: u64,
    pub style: UiTextStyleKey,
}

impl UiTextMeasureSizeKey {
    pub(crate) fn from_text_style(
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> Self {
        Self {
            content_hash: text_hash(text),
            style: UiTextStyleKey::from_style(style),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl UiTextShapePrewarmRequest {
    pub(crate) fn horizontal(text: impl Into<Arc<str>>, style: UiResolvedStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub(crate) fn from_layout_source(text: &str, style: UiResolvedStyle) -> Option<Self> {
        let text = layout_prewarm_text(text, style.rich_text_format)?;
        Some(Self { text, style })
    }

    fn to_shape_paragraph(&self) -> TextShapeParagraph {
        TextShapeParagraph::horizontal(
            Arc::clone(&self.text),
            text_style(&self.style),
            TextDirection::Auto,
            TextRange {
                start: 0,
                end: self.text.len(),
            },
        )
    }
}

fn layout_prewarm_text(text: &str, format: UiRichTextFormat) -> Option<Arc<str>> {
    if text.is_empty() {
        return None;
    }
    if matches!(format, UiRichTextFormat::Plain) {
        return Some(Arc::from(text));
    }

    let visible_text = parse_source_text(text, format.into()).text;
    (!visible_text.is_empty()).then(|| Arc::from(visible_text))
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextMeasureCache {
    measure_frame_dedup: TextFrameDedup<UiTextMeasureSizeKey, UiSize>,
    measure_cache: TextMeasureCache<UiTextMeasureSizeKey, UiSize>,
    text_layout_session: SharedTextLayoutSession,
    layout_frame_dedup: TextFrameDedup<UiTextMeasureKey, UiTextLayoutResolution>,
    layout_cache: TextLayoutCache<UiTextMeasureKey, UiTextLayoutResolution>,
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
            .map(UiTextShapePrewarmRequest::to_shape_paragraph)
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
        self.shape_prewarm_report.shaped_count = self
            .shape_prewarm_report
            .shaped_count
            .saturating_add(report.shaped_count);
        self.shape_prewarm_report.inserted_count = self
            .shape_prewarm_report
            .inserted_count
            .saturating_add(report.inserted_count);
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

    pub(crate) fn resolve_or_shape(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> UiTextLayoutResolution {
        let key = UiTextMeasureKey::from_request(request);
        let resolved_text = request.resolved_text();
        if let Some(resolution) = self
            .layout_frame_dedup
            .get(&key, resolved_text.as_str())
            .cloned()
        {
            return resolution;
        }

        let width_validity = TextLayoutWidthValidity::exact(request.frame.width);
        let resolution = if let Some(resolution) = self
            .layout_cache
            .get(&key, resolved_text.as_str(), request.frame.width)
            .cloned()
        {
            resolution
        } else {
            let resolution =
                resolve_text_layout_with_provider(request, &mut self.text_layout_session);
            self.layout_cache
                .insert(
                    key.clone(),
                    resolved_text.as_str(),
                    width_validity,
                    resolution,
                )
                .clone()
        };
        self.layout_frame_dedup
            .insert(key, resolved_text, resolution.clone());
        resolution
    }
}
