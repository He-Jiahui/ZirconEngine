#[cfg(test)]
use std::cell::Cell;
use std::ops::{Deref, DerefMut, Range};
use std::sync::Arc;

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::core::runtime::tasks::TaskPool;

use super::cache::{
    CompiledRichTextCacheReport, DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
    DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES, HardLineIndexCache, HardLineIndexCacheReport,
    ShapedRunCache, ShapedRunCacheLookupKey, ShapedRunCacheReport, TextDocumentKey,
};
use super::font::{
    FontCollectionRevision, FontCollectionService, FontCollectionSnapshot,
    shared_font_collection_service,
};
use super::model::TextShapingRequestDiagnostics;
use super::parallel::shape_pool::{
    TextParallelShapeBatchReport, TextShapeParagraph,
    shape_paragraphs_with_cache_in_font_collection,
};
use super::service::{
    shape_backend_request_at_stable_generation,
    shape_backend_request_at_stable_generation_in_font_collection,
};
use super::shaping::{
    TextShapeRunProvider, TextShapingOutcome, TextShapingWorkBudget, TextShapingWorkReport,
};
use super::{
    BackendShapeRequest, CompiledRichText, HardLine, RichTextFormat, RichTextParseError,
    RichTextParser, ShapedGlyphRun, TextRange, TextStyle, VerticalMode, hard_line_count_and_window,
};
use super::{TextLayoutGeometryBudget, TextLayoutGeometryOwner, TextLayoutGeometryViolation};

mod diagnostics;
mod table_work;

pub use diagnostics::TextLayoutFallbackReport;
pub(crate) use diagnostics::{
    TextLayoutGeometryRejectionReceipt, TextLayoutGeometryReport, TextLayoutSessionDiagnostics,
};
pub(crate) use table_work::TextTableLayoutWorkReport;

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

#[derive(Clone, Debug)]
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
}

impl PartialEq for SharedTextLayoutSession {
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
    /// Process-owner entrypoint for Editor-host and standalone text operations.
    /// Core-owned Runtime paths must use `new_with_font_collection` so every retained cache and
    /// renderer in that session observes the same font collection revision.
    pub(crate) fn new() -> Self {
        Self::new_with_font_collection(shared_font_collection_service())
    }

    pub(crate) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        Self::new_with_font_collection_and_geometry_budget(
            font_collection,
            TextLayoutGeometryBudget::default(),
        )
    }

    pub(crate) fn new_with_font_collection_and_geometry_budget(
        font_collection: Arc<FontCollectionService>,
        geometry_budget: TextLayoutGeometryBudget,
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
        }
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.shaped_runs.begin_frame(frame_index);
        self.shaping_work_report = TextShapingWorkReport::default();
        self.diagnostics = TextLayoutSessionDiagnostics::default();
        self.table_layout_work_report = TextTableLayoutWorkReport::default();
    }

    pub(crate) fn finish_frame(&mut self) {
        self.table_layout_work_report.publish_profile_counters();
        self.shaped_runs.finish_frame();
    }

    pub(crate) fn clear(&mut self) {
        self.rich_text_parser.clear_compiled_cache();
        self.shaped_runs.clear();
        self.hard_line_index.clear();
    }

    pub(crate) fn compile_rich_text(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Result<Arc<CompiledRichText>, RichTextParseError> {
        self.rich_text_parser.compile(markup, format)
    }

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

    pub(crate) fn record_layout_error(&mut self, error: &TextLayoutError) {
        self.diagnostics.record_layout_error(error);
    }

    pub(crate) fn retained_hard_line_count_and_window(
        &mut self,
        source: Arc<str>,
        document_key: TextDocumentKey,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        self.hard_line_index
            .count_and_window(document_key, source, range)
    }

    pub(crate) fn unretained_hard_line_count_and_window(
        &mut self,
        text: &str,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        self.hard_line_index.record_unkeyed_bypass();
        hard_line_count_and_window(text, range)
    }

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
        if let Some(run) = self.shaped_runs.get_with_lookup(&lookup, request.text) {
            if lookup_generation == self.font_collection.generation() {
                return TextShapingOutcome::Ready(run);
            }
            let failure = crate::text::shaping::TextShapingFailure::font_generation_changed();
            self.diagnostics.record_deferred_failure(&failure);
            return TextShapingOutcome::Deferred(failure);
        }
        self.shaping_work_report
            .record_synchronous_request(self.shaping_work_budget, request.text.len());
        let outcome = shape_request_with_generation_outcome_in_font_collection(
            request,
            &self.font_collection,
        );
        self.consume_shaping_outcome(&lookup, lookup_generation, outcome)
    }

    fn consume_shaping_outcome(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
        lookup_generation: u64,
        outcome: TextShapingOutcome<GenerationTaggedShapedRun>,
    ) -> TextShapingOutcome {
        match outcome {
            TextShapingOutcome::Ready(tagged) => {
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

pub(super) fn shape_request_with_generation_outcome(
    request: BackendShapeRequest<'_>,
) -> TextShapingOutcome<GenerationTaggedShapedRun> {
    TextShapingOutcome::from_shape_result(
        try_shape_request_through_canonical_service_with_generation(request),
    )
}

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
#[path = "layout_session/work_budget.rs"]
mod work_budget_tests;

#[cfg(test)]
#[path = "layout_session/tests.rs"]
mod tests;
