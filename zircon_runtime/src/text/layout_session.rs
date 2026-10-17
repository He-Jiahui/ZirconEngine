//! 文本布局会话持有字体集合、保留缓存及逐帧工作和诊断报告。
//!
//! 调用方在帧边界推进缓存与报告；成形结果携带字体代次，只有代次仍匹配才进入缓存。
//! `new` 默认入口使用进程字体集合；上下文创建入口另传会话租约，克隆共享其租约家族。
//! 竖排作用域只调整指定 trait 入口的请求方向，布局和几何准入仍由上层负责。

#[cfg(test)]
use std::cell::Cell;
use std::ops::{Deref, DerefMut, Range};
use std::sync::Arc;

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::core::runtime::tasks::TaskPool;

use super::cache::{
    CompiledRichTextCacheReport, HardLineIndexCache, HardLineIndexCacheReport, ShapedRunCache,
    ShapedRunCacheLookupKey, ShapedRunCacheReport, TextDocumentKey,
    DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
use super::context::{TextRuntimeSessionLease, TextSessionId};
use super::font::{
    shared_font_collection_service, FontCollectionRevision, FontCollectionService,
    FontCollectionSnapshot,
};
use super::model::TextShapingRequestDiagnostics;
use super::parallel::shape_pool::{
    shape_paragraphs_with_cache_in_font_collection, TextParallelShapeBatchReport,
    TextShapeParagraph,
};
use super::service::{
    shape_backend_request_at_stable_generation,
    shape_backend_request_at_stable_generation_in_font_collection,
};
use super::shaping::{
    TextShapeRunProvider, TextShapingOutcome, TextShapingWorkBudget, TextShapingWorkReport,
};
use super::{
    hard_line_count_and_window, BackendShapeRequest, CompiledRichText, HardLine, RichTextFormat,
    RichTextParseError, RichTextParser, ShapedGlyphRun, TextRange, TextStyle, VerticalMode,
};
use super::{TextLayoutGeometryBudget, TextLayoutGeometryOwner, TextLayoutGeometryViolation};

#[path = "layout_session/diagnostics.rs"]
mod diagnostics;
#[path = "layout_session/table_work.rs"]
mod table_work;

pub use diagnostics::TextLayoutFallbackReport;
pub(crate) use diagnostics::{
    TextLayoutGeometryRejectionReceipt, TextLayoutGeometryReport, TextLayoutSessionDiagnostics,
};
pub(crate) use table_work::TextTableLayoutWorkReport;

/// 后端结果及其字体代次、请求诊断的拥有式回执。
///
/// `run` 通过引用计数保留字形数据；代次在消费结果时再次比对，诊断在缓存准入前记录。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct GenerationTaggedShapedRun {
    pub(super) run: Arc<ShapedGlyphRun>,
    pub(super) font_generation: u64,
    pub(super) request_diagnostics: TextShapingRequestDiagnostics,
}

#[cfg(test)]
thread_local! {
    static CURRENT_THREAD_SESSION_CONSTRUCTION_COUNT: Cell<u64> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn current_thread_text_layout_session_construction_count() -> u64 {
    CURRENT_THREAD_SESSION_CONSTRUCTION_COUNT.get()
}

#[cfg(test)]
fn record_text_layout_session_construction() {
    CURRENT_THREAD_SESSION_CONSTRUCTION_COUNT.set(
        CURRENT_THREAD_SESSION_CONSTRUCTION_COUNT
            .get()
            .saturating_add(1),
    );
}

/// 会话内的文本缓存、字体集合引用及布局工作报告所有者。
///
/// # 使用与生命周期
/// UI 缓存持有此对象并调用帧边界方法；上下文入口可附带会话租约。
/// 字体集合由引用计数共享，租约家族的最后一份引用释放时结束该家族的活动计数。
/// 克隆重建富文本解析器，复制其余缓存结构、报告、预算和当前模式，并共享字体集合与租约家族。
///
/// # 状态边界
/// `begin_frame` 重置本帧报告；`finish_frame` 将表格计数交给采集入口并整理成形缓存；
/// `clear` 清理保留缓存，不重置这里的工作、诊断或表格报告。
#[derive(Debug)]
pub(crate) struct SharedTextLayoutSession {
    font_collection: Arc<FontCollectionService>,
    rich_text_parser: Arc<RichTextParser>,
    shaped_runs: ShapedRunCache,
    hard_line_index: HardLineIndexCache,
    geometry_budget: TextLayoutGeometryBudget,
    shaping_work_budget: TextShapingWorkBudget,
    shaping_work_report: TextShapingWorkReport,
    diagnostics: TextLayoutSessionDiagnostics,
    table_layout_work_report: TextTableLayoutWorkReport,
    vertical_mode: Option<VerticalMode>,
    _context_session_lease: Option<TextRuntimeSessionLease>,
}

impl Clone for SharedTextLayoutSession {
    /// 复制会话状态，但为富文本编译缓存创建独立解析器；不申请新的上下文会话身份。
    fn clone(&self) -> Self {
        Self {
            font_collection: Arc::clone(&self.font_collection),
            // A cloned session is a new cache owner. Keep the immutable layout state and lease
            // family, but never let clear/telemetry on one surface reach another parser cache.
            rich_text_parser: Arc::new(RichTextParser::default()),
            shaped_runs: self.shaped_runs.clone(),
            hard_line_index: self.hard_line_index.clone(),
            geometry_budget: self.geometry_budget,
            shaping_work_budget: self.shaping_work_budget,
            shaping_work_report: self.shaping_work_report,
            diagnostics: self.diagnostics,
            table_layout_work_report: self.table_layout_work_report,
            vertical_mode: self.vertical_mode,
            _context_session_lease: self._context_session_lease.clone(),
        }
    }
}

impl PartialEq for SharedTextLayoutSession {
    // 比较列出的布局状态；解析器对象和上下文租约不参与此相等关系。
    fn eq(&self, other: &Self) -> bool {
        self.font_collection == other.font_collection
            && self.shaped_runs == other.shaped_runs
            && self.hard_line_index == other.hard_line_index
            && self.geometry_budget == other.geometry_budget
            && self.shaping_work_budget == other.shaping_work_budget
            && self.shaping_work_report == other.shaping_work_report
            && self.diagnostics == other.diagnostics
            && self.table_layout_work_report == other.table_layout_work_report
            && self.vertical_mode == other.vertical_mode
    }
}

impl Default for SharedTextLayoutSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTextLayoutSession {
    /// 创建未附带上下文会话身份的独立会话；默认构造也使用此入口。
    /// Process-owner entrypoint for Editor-host and standalone text operations.
    /// Core-owned Runtime paths must use `new_with_font_collection` so every retained cache and
    /// renderer in that session observes the same font collection revision.
    pub(crate) fn new() -> Self {
        Self::new_with_font_collection(shared_font_collection_service())
    }

    /// 使用调用方提供的字体集合和默认几何预算创建会话。
    ///
    /// # 参数与返回
    /// `font_collection` 的共享所有权移入新会话；返回值拥有独立缓存且没有上下文租约。
    pub(crate) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        Self::new_with_font_collection_and_geometry_budget(
            font_collection,
            TextLayoutGeometryBudget::default(),
        )
    }

    /// 为已获上下文准入的会话绑定字体集合与租约。
    ///
    /// # 参数、返回与生命周期
    /// 两个拥有式参数一并移入新会话；返回值持有租约，不借用上下文本体。
    /// 租约克隆延续同一家族，最终释放由租约家族负责；准入和活动状态检查由调用方完成。
    pub(super) fn new_with_font_collection_and_context_lease(
        font_collection: Arc<FontCollectionService>,
        context_session_lease: TextRuntimeSessionLease,
    ) -> Self {
        Self::new_with_font_collection_geometry_budget_and_context_lease(
            font_collection,
            TextLayoutGeometryBudget::default(),
            Some(context_session_lease),
        )
    }

    /// 使用指定字体集合和几何预算快照创建无上下文租约的会话。
    ///
    /// 预算按值保存，供布局及准入调用方读取；构造本身不验证任何布局输出。
    pub(crate) fn new_with_font_collection_and_geometry_budget(
        font_collection: Arc<FontCollectionService>,
        geometry_budget: TextLayoutGeometryBudget,
    ) -> Self {
        Self::new_with_font_collection_geometry_budget_and_context_lease(
            font_collection,
            geometry_budget,
            None,
        )
    }

    fn new_with_font_collection_geometry_budget_and_context_lease(
        font_collection: Arc<FontCollectionService>,
        geometry_budget: TextLayoutGeometryBudget,
        context_session_lease: Option<TextRuntimeSessionLease>,
    ) -> Self {
        #[cfg(test)]
        record_text_layout_session_construction();
        Self {
            font_collection,
            rich_text_parser: Arc::new(RichTextParser::default()),
            shaped_runs: ShapedRunCache::with_limits(
                DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
                DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
            ),
            hard_line_index: HardLineIndexCache::default(),
            geometry_budget,
            shaping_work_budget: TextShapingWorkBudget::default(),
            shaping_work_report: TextShapingWorkReport::default(),
            diagnostics: TextLayoutSessionDiagnostics::default(),
            table_layout_work_report: TextTableLayoutWorkReport::default(),
            vertical_mode: None,
            _context_session_lease: context_session_lease,
        }
    }

    /// 开始指定帧：推进成形缓存，并重置工作、诊断及表格报告；保留缓存条目。
    ///
    /// `frame_index` 交给缓存标识本帧使用情况，不是文档修订号。
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.shaped_runs.begin_frame(frame_index);
        self.shaping_work_report = TextShapingWorkReport::default();
        self.diagnostics = TextLayoutSessionDiagnostics::default();
        self.table_layout_work_report = TextTableLayoutWorkReport::default();
    }

    /// 将当前表格计数交给采集入口，并执行成形缓存的帧尾整理；报告保留至下一次帧开始。
    pub(crate) fn finish_frame(&mut self) {
        self.table_layout_work_report.publish_profile_counters();
        self.shaped_runs.finish_frame();
    }

    /// 清空富文本编译、成形及硬行索引缓存；工作和诊断报告的帧边界另行推进。
    pub(crate) fn clear(&mut self) {
        self.rich_text_parser.clear_compiled_cache();
        self.shaped_runs.clear();
        self.hard_line_index.clear();
    }

    /// 返回附带租约的会话身份；独立构造的会话返回 `None`，不会临时分配身份。
    pub(crate) fn text_session_id(&self) -> Option<TextSessionId> {
        self._context_session_lease
            .as_ref()
            .map(TextRuntimeSessionLease::id)
    }

    /// 编译指定格式的标记文本，并由本会话解析器复用编译缓存。
    ///
    /// # 参数与返回
    /// `markup` 仅在调用期间借用，`format` 选择解析规则；成功返回可独立持有的共享编译产物，
    /// 失败透传解析错误。清理会话缓存不会撤销调用方已持有的产物引用。
    pub(crate) fn compile_rich_text(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Result<Arc<CompiledRichText>, RichTextParseError> {
        self.rich_text_parser.compile(markup, format)
    }

    /// 仅查询给定标记文本与格式的编译缓存；返回共享产物或 `None`，不触发编译。
    pub(crate) fn lookup_compiled_rich_text(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Option<Arc<CompiledRichText>> {
        self.rich_text_parser.lookup_compiled(markup, format)
    }

    pub(crate) fn compiled_rich_text_cache_report(&self) -> CompiledRichTextCacheReport {
        self.rich_text_parser.compiled_cache_report()
    }

    pub(crate) fn take_compiled_rich_text_cache_report(&self) -> CompiledRichTextCacheReport {
        self.rich_text_parser.take_compiled_cache_report()
    }

    pub(crate) fn font_database_generation(&self) -> u64 {
        self.font_collection.generation()
    }

    pub(crate) fn font_collection_revision(&self) -> FontCollectionRevision {
        self.font_collection.revision()
    }

    /// 取得此会话字体集合的拥有式快照；快照自身保留所引用的字体数据库资源。
    pub(crate) fn font_collection_snapshot(&self) -> FontCollectionSnapshot {
        self.font_collection.collection_snapshot()
    }

    pub(crate) fn cache_report(&self) -> ShapedRunCacheReport {
        self.shaped_runs.report()
    }

    pub(crate) fn hard_line_index_report(&self) -> HardLineIndexCacheReport {
        self.hard_line_index.report()
    }

    pub(crate) const fn shaping_work_report(&self) -> TextShapingWorkReport {
        self.shaping_work_report
    }

    pub(crate) const fn diagnostics_report(&self) -> TextLayoutSessionDiagnostics {
        self.diagnostics
    }

    pub(crate) const fn geometry_budget(&self) -> TextLayoutGeometryBudget {
        self.geometry_budget
    }

    pub(crate) const fn geometry_report(&self) -> TextLayoutGeometryReport {
        self.diagnostics.geometry
    }

    pub(crate) const fn table_layout_work_report(&self) -> TextTableLayoutWorkReport {
        self.table_layout_work_report
    }

    pub(crate) fn record_table_layout_attempt(&mut self, source_bytes: usize, cell_count: usize) {
        self.table_layout_work_report
            .record_layout_attempt(source_bytes, cell_count);
    }

    pub(crate) fn record_table_layout_tracks(&mut self, column_count: usize, row_count: usize) {
        self.table_layout_work_report
            .record_tracks(column_count, row_count);
    }

    pub(crate) fn record_table_preferred_cell_layout(&mut self, source_bytes: usize) {
        self.table_layout_work_report
            .record_preferred_cell_layout(source_bytes);
    }

    pub(crate) fn record_table_final_cell_layout(&mut self, source_bytes: usize) {
        self.table_layout_work_report
            .record_final_cell_layout(source_bytes);
    }

    pub(crate) fn record_table_layout_output(&mut self, line_count: usize, box_count: usize) {
        self.table_layout_work_report
            .record_output(line_count, box_count);
    }

    /// 记录本次几何拒绝并构造供调用方返回的布局错误。
    ///
    /// # 参数与返回
    /// `owner` 和 `violation` 标识失败阶段及超界值；`source_range` 是可选原文定位，
    /// `work_units` 是调用方提供的已观测工作量。返回几何超限错误，同时更新几何报告。
    /// 回退分类由 `record_layout_error` 独立记录；此入口不自动增加回退计数。
    pub(crate) fn reject_geometry(
        &mut self,
        owner: TextLayoutGeometryOwner,
        violation: TextLayoutGeometryViolation,
        source_range: Option<(u32, u32)>,
        work_units: usize,
    ) -> TextLayoutError {
        self.diagnostics
            .record_geometry_rejection(owner, violation, source_range, work_units);
        TextLayoutError::GeometryTooLarge
    }

    /// 将调用方最终处理的布局错误计入回退分类；与几何拒绝回执的记录相互独立。
    pub(crate) fn record_layout_error(&mut self, error: &TextLayoutError) {
        self.diagnostics.record_layout_error(error);
    }

    /// 按文档键查询硬行总数和行序号窗口，允许缓存保留源文本及完整行索引。
    ///
    /// # 参数、返回与生命周期
    /// `source` 移入共享文本引用，`document_key` 关联文档文本；`range` 是行序号范围。
    /// 返回总行数和拥有式窗口；缓存是否保留完整索引还受其容量与字节限制。
    pub(crate) fn retained_hard_line_count_and_window(
        &mut self,
        source: Arc<str>,
        document_key: TextDocumentKey,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        self.hard_line_index
            .count_and_window(document_key, source, range)
    }

    /// 对临时借用文本求硬行总数与行序号窗口，并记录无键旁路。
    ///
    /// 返回拥有式窗口；不把借用文本或完整行索引保留在会话中。
    pub(crate) fn unretained_hard_line_count_and_window(
        &mut self,
        text: &str,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        self.hard_line_index.record_unkeyed_bypass();
        hard_line_count_and_window(text, range)
    }

    /// 在本会话字体集合和成形缓存上预热段落，并合并工作及诊断回执。
    ///
    /// # 参数与返回
    /// `pool`、`paragraphs` 在调用期间借用；`chunk_size` 交给批处理组织任务。
    /// 返回拥有式批报告，缓存及本帧报告可能已经更新；工作预算用于分类观测工作，
    /// 不在此授权切分语义范围或把失败替换为空字形结果。
    pub(crate) fn prewarm_horizontal_paragraphs(
        &mut self,
        pool: &TaskPool,
        paragraphs: &[TextShapeParagraph],
        chunk_size: usize,
    ) -> TextParallelShapeBatchReport {
        let report = shape_paragraphs_with_cache_in_font_collection(
            pool,
            &mut self.shaped_runs,
            paragraphs,
            chunk_size,
            self.shaping_work_budget,
            &self.font_collection,
        );
        self.shaping_work_report.merge(report.shaping_work);
        self.diagnostics.merge_shaping(report.shaping_diagnostics);
        report
    }

    /// 按显式横排请求取得缓存或后端结果，不读取当前竖排作用域模式。
    ///
    /// # 参数与返回
    /// `text`、`style` 仅在请求期间借用；`direction` 与 `source_range` 指定方向和原文定位。
    /// 返回拥有字形引用的就绪结果，或保留类型信息的延期／失败；非法请求可在后端调用前失败。
    pub(crate) fn shape_horizontal_range(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
    ) -> TextShapingOutcome {
        self.resolve_or_shape_outcome(BackendShapeRequest::horizontal(
            text,
            style,
            direction,
            source_range,
        ))
    }

    /// 按传入的 `vertical_mode` 构造启用字距调整的竖排请求。
    ///
    /// 文本、样式、方向、原文范围及结果所有权同横排入口；模式参数优先于会话作用域。
    pub(crate) fn shape_vertical_range(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
    ) -> TextShapingOutcome {
        self.resolve_or_shape_outcome(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            true,
        ))
    }

    /// 暂存原模式并独占借用会话，供通用成形调用在竖排布局期间改道。
    ///
    /// # 参数、返回与约束
    /// `vertical_mode` 保存到会话；返回守卫可解引用为会话，释放时恢复进入前的模式。
    /// 该状态仅改变 trait 的横排带字距入口；固有横排方法及显式竖排模式参数不受影响。
    /// 参考 [`TextShapeRunProvider::shape_horizontal_range_with_kerning`]。
    pub(crate) fn vertical_scope(
        &mut self,
        vertical_mode: VerticalMode,
    ) -> VerticalTextLayoutScope<'_> {
        let previous_mode = self.vertical_mode.replace(vertical_mode);
        VerticalTextLayoutScope {
            session: self,
            previous_mode,
        }
    }

    // 先规范化与校验，再以本集合及代次查缓存；前置失败尚未记录后端工作。
    fn resolve_or_shape_outcome(&mut self, request: BackendShapeRequest<'_>) -> TextShapingOutcome {
        let canonical_request = match request.canonicalized() {
            Ok(request) => request,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let request = canonical_request.request();
        if !request.style.font_size.is_finite() || request.style.font_size <= 0.0 {
            return TextShapingOutcome::failed(TextLayoutError::InvalidFontSize);
        }
        let lookup = ShapedRunCacheLookupKey::from_request_in_font_collection(
            &request,
            self.font_collection.collection_id(),
            self.font_collection.generation(),
        );
        let lookup_generation = lookup.font_database_generation();
        // 命中不重复计后端工作；命中后代次已变则返回延期，避免交出过时代次的结果。
        if let Some(run) = self.shaped_runs.get_with_lookup(&lookup, request.text) {
            if lookup_generation == self.font_collection.generation() {
                return TextShapingOutcome::Ready(run);
            }
            let failure = crate::text::shaping::TextShapingFailure::font_generation_changed();
            self.diagnostics.record_deferred_failure(&failure);
            return TextShapingOutcome::Deferred(failure);
        }
        // 未命中先记录尝试的工作量，因此后端延期或失败也可留下本帧工作回执。
        self.shaping_work_report
            .record_synchronous_request(self.shaping_work_budget, request.text.len());
        let outcome = shape_request_with_generation_outcome_in_font_collection(
            request,
            &self.font_collection,
        );
        self.consume_shaping_outcome(&lookup, lookup_generation, outcome)
    }

    // 将后端回执并入会话：只有查询、产物和当前字体代次一致的就绪结果才写入缓存。
    fn consume_shaping_outcome(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
        lookup_generation: u64,
        outcome: TextShapingOutcome<GenerationTaggedShapedRun>,
    ) -> TextShapingOutcome {
        match outcome {
            TextShapingOutcome::Ready(tagged) => {
                // 先记录观测到的就绪诊断；随后代次不符仍会另记延期且不缓存。
                self.diagnostics
                    .record_ready_run(&tagged.run, tagged.request_diagnostics);
                if tagged.font_generation == lookup_generation
                    && lookup_generation == self.font_collection.generation()
                {
                    let key = self.shaped_runs.own_lookup_key(lookup);
                    TextShapingOutcome::Ready(self.shaped_runs.insert_ready(key, tagged.run))
                } else {
                    let failure =
                        crate::text::shaping::TextShapingFailure::font_generation_changed();
                    self.diagnostics.record_deferred_failure(&failure);
                    TextShapingOutcome::Deferred(failure)
                }
            }
            TextShapingOutcome::Deferred(error) => {
                self.diagnostics.record_deferred_failure(&error);
                TextShapingOutcome::Deferred(error)
            }
            TextShapingOutcome::Failed(error) => {
                self.diagnostics.record_terminal_failure(&error);
                TextShapingOutcome::Failed(error)
            }
        }
    }
}

/// 使用进程字体集合执行稳定代次请求；成功时把字形、代次和诊断一起转为拥有式回执。
fn try_shape_request_through_canonical_service_with_generation(
    request: BackendShapeRequest<'_>,
) -> Result<GenerationTaggedShapedRun, crate::text::shaping::TextShapingFailure> {
    shape_backend_request_at_stable_generation(
        request,
        |run, _, font_generation, request_diagnostics| GenerationTaggedShapedRun {
            run: Arc::new(run),
            font_generation,
            request_diagnostics,
        },
    )
}

/// 在借用的指定集合中执行同一稳定代次流程；返回回执不借用请求文本或集合参数。
fn try_shape_request_through_canonical_service_with_generation_in_font_collection(
    request: BackendShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
) -> Result<GenerationTaggedShapedRun, crate::text::shaping::TextShapingFailure> {
    shape_backend_request_at_stable_generation_in_font_collection(
        request,
        font_collection,
        |run, _, font_generation, request_diagnostics| GenerationTaggedShapedRun {
            run: Arc::new(run),
            font_generation,
            request_diagnostics,
        },
    )
}

/// 将进程字体集合请求的成功／失败转换为带代次的类型化结果。
pub(super) fn shape_request_with_generation_outcome(
    request: BackendShapeRequest<'_>,
) -> TextShapingOutcome<GenerationTaggedShapedRun> {
    TextShapingOutcome::from_shape_result(
        try_shape_request_through_canonical_service_with_generation(request),
    )
}

/// 在指定集合中成形并保留代次回执。
///
/// 请求及集合只在调用期间借用；返回就绪值拥有字形引用，延期或失败保持原错误分类。
pub(super) fn shape_request_with_generation_outcome_in_font_collection(
    request: BackendShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
) -> TextShapingOutcome<GenerationTaggedShapedRun> {
    TextShapingOutcome::from_shape_result(
        try_shape_request_through_canonical_service_with_generation_in_font_collection(
            request,
            font_collection,
        ),
    )
}

#[cfg(test)]
pub(super) fn shape_request_outcome(request: BackendShapeRequest<'_>) -> TextShapingOutcome {
    shape_request_with_generation_outcome(request).map(|tagged| tagged.run)
}

// 通用成形器通过此 trait 进入；只有横排带字距方法查询会话的临时竖排模式。
impl TextShapeRunProvider for SharedTextLayoutSession {
    fn font_collection_revision(&self) -> FontCollectionRevision {
        SharedTextLayoutSession::font_collection_revision(self)
    }

    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        match self.vertical_mode {
            Some(vertical_mode) => {
                self.resolve_or_shape_outcome(BackendShapeRequest::vertical_with_kerning(
                    text,
                    style,
                    direction,
                    source_range,
                    vertical_mode,
                    include_kerning,
                ))
            }
            None => self.resolve_or_shape_outcome(BackendShapeRequest::horizontal_with_kerning(
                text,
                style,
                direction,
                source_range,
                include_kerning,
            )),
        }
    }

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.resolve_or_shape_outcome(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        ))
    }
}

/// 保存临时竖排模式的独占借用守卫。
///
/// # 生命周期
/// 守卫通过解引用转发会话操作；销毁时恢复进入前的模式，包括返回错误而提前退出的路径。
/// 它不拥有新会话或新租约；内部可变借用结束后调用方才可再次直接访问原会话。
pub(crate) struct VerticalTextLayoutScope<'a> {
    session: &'a mut SharedTextLayoutSession,
    previous_mode: Option<VerticalMode>,
}

impl Deref for VerticalTextLayoutScope<'_> {
    type Target = SharedTextLayoutSession;

    fn deref(&self) -> &Self::Target {
        self.session
    }
}

impl DerefMut for VerticalTextLayoutScope<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session
    }
}

impl Drop for VerticalTextLayoutScope<'_> {
    fn drop(&mut self) {
        self.session.vertical_mode = self.previous_mode;
    }
}

#[cfg(test)]
#[path = "layout_session/tests/work_budget.rs"]
mod work_budget_tests;

#[cfg(test)]
#[path = "layout_session/tests/cases.rs"]
mod tests;
