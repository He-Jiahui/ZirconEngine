//! UI 文本测量与布局的保留缓存门面，供 Surface 与独立抽取入口复用。
//!
//! 调用方负责帧开始与结束；本模块拥有结果缓存、帧内去重及纯文本解析驻留，
//! 会话提供字体代次、成形和诊断。构造与租约见 `context`，解析驻留见 `retained_document`。
//! viewport 探测也可能更新解析缓存和索引；结果命中不表示本轮执行过成形。

use crate::core::framework::text::TextDirection;
use crate::core::runtime::tasks::TaskPool;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
#[cfg(feature = "profiling")]
use crate::text::CompiledRichTextCacheReport;
use crate::text::{
    cache::{
        HardLineIndexCacheReport, ShapedRunCacheReport, TextFrameDedup, TextFrameDedupReport,
        TextLayoutCache, TextLayoutCacheReport, TextLayoutWidthValidity, TextMeasureCache,
        TextMeasureCacheReport, DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY,
        DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
    },
    from_compiled_rich_semantic_projection, has_multiple_hard_lines,
    layout::resolved_text_spans,
    parallel::shape_pool::{TextParallelShapeBatchReport, TextShapeParagraph},
    text_style, EphemeralCacheHash, RichSemanticProjection, RichTextFormat,
    SharedTextLayoutSession, TextDocumentKey, TextRange, TextStyle, VerticalMode,
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
    resolution_from_layout, resolve_text_layout_with_provider_and_parsed_outcome,
    resolve_text_layout_with_provider_outcome, UiTextLayoutRequest, UiTextLayoutResolution,
    UiTextStyleKey,
};
use super::rich_text::{parse_source_text_with_provider, UiParsedText};
use super::shaper::measure_unwrapped_text_height_with_provider;

#[path = "measure_cache/context.rs"]
mod context;
#[path = "measure_cache/retained_document.rs"]
mod retained_document;

use retained_document::RetainedPlainTextDocumentCache;
#[cfg(test)]
use retained_document::RETAINED_PLAIN_DOCUMENT_MAX_BYTES;

/// 布局几何、视口、样式和字体代次共同限定的查找键；摘要不能替代缓存对原文的核对。
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

/// 尺寸测量键不带布局 frame；原文、样式和会话字体代次共同限定结果复用。
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
#[path = "measure_cache/tests/generation_key_tests.rs"]
mod generation_key_tests;

/// 保存 frame 的浮点位模式；正负零等价，其余位值包括非有限值照存。
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

/// 拥有待预热段落，允许批处理期间独立持有文本及样式。
///
/// 显式横排构造器生成横排段落；从布局源生成时仍保留实际竖排信息，
/// 不保证每个请求都横排，也不代替布局结果缓存。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextShapePrewarmRequest {
    paragraphs: Vec<TextShapeParagraph>,
}

impl UiTextShapePrewarmRequest {
    /// 构造显式横排的预热输入。
    ///
    /// # 参数与返回
    /// `text` 转为共享拥有的文本，`style` 随请求持有；返回可独立传给预热批次的段落集合。
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

    /// 按源格式和写作模式生成段落；空源、解析或富文本跨度校验失败时返回 None。
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

/// Surface 可长期持有的文本会话与两层结果缓存。
///
/// # 使用与生命周期
/// 拥有尺寸/布局持久缓存、各自的帧内去重及解析驻留；字体代次来自会话。
/// 克隆复制缓存和报告结构，会话重建解析器但共享字体集合和已有租约家族。
/// 上下文构造入口检查准入；测量调用不逐次检查上下文是否仍为 Active。
/// 持久结果缓存按条目数限额，字节报告为估计值；不等于整个文本管线的内存上限。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextMeasureCache {
    text_runtime_context_id: Option<crate::text::TextRuntimeContextId>,
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

impl UiTextMeasureCache {
    /// 返回会话当前字体数据库代次，供结果键与抽取缓存失效判断使用。
    pub(crate) fn font_database_generation(&self) -> u64 {
        self.text_layout_session.font_database_generation()
    }

    /// 根据源和样式生成拥有的预热输入；不可解析或无有效段落时返回 None。
    pub(crate) fn shape_prewarm_request(
        &self,
        text: &str,
        style: UiResolvedStyle,
    ) -> Option<UiTextShapePrewarmRequest> {
        UiTextShapePrewarmRequest::from_layout_source(text, style, &self.text_layout_session)
    }

    /// 用本会话解析器处理借入源；返回拥有的解析结果，错误保留原类型。
    pub(crate) fn parse_source_text(
        &self,
        source_markup: &str,
        format: RichTextFormat,
    ) -> Result<UiParsedText, crate::core::framework::text::TextLayoutError> {
        parse_source_text_with_provider(source_markup, format, &self.text_layout_session)
    }

    /// 借用本会话编译富文本并返回拥有的语义投影；编译失败映射为 None。
    /// 供未产生渲染命令的语义读取复用，不依赖可见布局结果。
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

    /// 返回配对的数据库代次与共享数据库快照，延长旧数据库寿命。
    /// 快照仍持有集合服务，不阻止服务以后发布新代次。
    pub(crate) fn font_collection_snapshot(&self) -> crate::text::font::FontCollectionSnapshot {
        self.text_layout_session.font_collection_snapshot()
    }

    /// 清除相关缓存内容；各报告的归零边界由对应缓存/会话实现决定。
    /// 不把帧序号、未缓存解析计数、预热报告及解析驻留的历史统计统一归零。
    pub(crate) fn clear(&mut self) {
        self.measure_frame_dedup.clear();
        self.measure_cache.clear();
        self.text_layout_session.clear();
        self.layout_frame_dedup.clear();
        self.layout_cache.clear();
        self.retained_plain_documents.clear();
    }

    /// 开始新帧：饱和递增帧序号，重置各帧去重/报告及本帧预热、未缓存解析计数。
    /// 持久结果和解析驻留可跨帧复用；与帧尾入口配对由调用方负责。
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

    /// 执行缓存与会话帧尾整理；调用采集入口不保证遥测已实际采集。
    pub(crate) fn finish_frame(&mut self) {
        self.measure_cache.finish_frame();
        self.text_layout_session.finish_frame();
        self.layout_cache.finish_frame();
    }

    /// 借用并原地更新命令片段的文本 artifact，复用本会话，不返回另一份命令列表。
    pub(crate) fn prepare_render_command_text_artifacts(
        &mut self,
        commands: &mut [UiRenderCommand],
    ) {
        super::rich_text::prepare_render_command_text_artifacts_with_provider(
            commands,
            &mut self.text_layout_session,
        );
    }

    /// 返回布局结果缓存的 miss_count，不等同于真实后端成形次数。
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

    /// 将请求中的拥有段落展平交给会话预热，并累计本帧批次报告。
    ///
    /// # 参数与返回
    /// `pool` 借用执行池，`requests` 借用请求切片，`chunk_size` 传给批处理；
    /// 返回本次报告。方法名不限制请求的实际写作模式，不生成完整布局。
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

    /// 本帧多批计数按饱和和合并，批大小/并行度取最大值，工作报告按自身契约合并。
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

    /// 查询文本尺寸，先查帧内去重，再查持久结果。
    ///
    /// # 参数与返回
    /// 借用完整文本及样式，返回尺寸值；空文本或非 Ready 结果返回默认尺寸。
    /// 非 Ready 会记录诊断但不写入这两个结果缓存；会话内部状态仍可能变化。
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

    /// 复用会话查询不折行高度；返回 None 时由调用方选择完整尺寸测量。
    pub(crate) fn measure_unwrapped_text_height(
        &mut self,
        text: &str,
        style: &zircon_runtime_interface::ui::surface::UiResolvedStyle,
    ) -> Option<f32> {
        measure_unwrapped_text_height_with_provider(text, style, &mut self.text_layout_session)
    }

    /// 判定严格硬行子集；资格要求文档键、无 preedit、viewport 及 Plain/HorizontalTb/NoWrap/Clip。
    /// 本查询访问解析驻留和硬行索引；返回 false 不表示完全没有缓存副作用。
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

    /// 返回布局解析结果；非 Ready 时用同一会话构造回退结果。
    /// 借用请求并返回拥有的结果，不把回退写入本门面的两级布局结果缓存。
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

    /// 尝试复用或生成布局，保留 Deferred/Failed 供需要区分状态的调用方处理。
    /// 只有 Ready 可入布局结果缓存；严格 viewport 子集只入帧内去重，绕过持久布局缓存。
    /// 请求原文仍参与命中校验，摘要或文档键不能单独证明源一致。
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

    /// 仅为符合虚拟化资格且有多硬行的请求取解析；探测与布局共享索引。
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

    /// 有键时走精确源比较的驻留入口；无键仅临时 Plain 解析，不构成 viewport 资格。
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
