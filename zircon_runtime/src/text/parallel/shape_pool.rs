//! Parallel paragraph shaping batch helpers.

use std::{collections::HashMap, sync::Arc, time::Instant};

#[cfg(any(test, feature = "profiling"))]
use std::collections::HashSet;

use crate::core::framework::text::TextDirection;
use crate::core::runtime::tasks::{TaskPool, parallel_for};
use crate::text::cache::{ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheLookupKey};
use crate::text::font::FontCollectionService;
#[cfg(test)]
use crate::text::font::shared_font_collection_service;
use crate::text::layout_session::{
    GenerationTaggedShapedRun, shape_request_with_generation_outcome_in_font_collection,
};
use crate::text::shaping::{
    TextShapingDiagnosticsReport, TextShapingFailure, TextShapingOutcome, TextShapingWorkBudget,
    TextShapingWorkReport,
};
use crate::text::{BackendShapeRequest, EphemeralCacheHash, VerticalMode};
use crate::text::{TextRange, TextStyle};

const TEXT_SHAPE_PARALLEL_MIN_JOBS: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapeParagraph {
    text: Arc<str>,
    style: TextStyle,
    base_direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
    vertical_mode: Option<VerticalMode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextParallelShapeBatchReport {
    pub(crate) requested_count: usize,
    pub(crate) cache_hit_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) batch_duplicate_count: usize,
    pub(crate) pending_lookup_candidate_count: usize,
    pub(crate) shaped_count: usize,
    pub(crate) inserted_count: usize,
    pub(crate) invalid_request_count: usize,
    pub(crate) generation_deferred_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) inline_batch_count: usize,
    pub(crate) parallel_join_count: usize,
    pub(crate) caller_wait_nanos: u64,
    pub(crate) chunk_size: usize,
    pub(crate) worker_parallelism: usize,
    pub(crate) shaping_work: TextShapingWorkReport,
    pub(crate) shaping_diagnostics: TextShapingDiagnosticsReport,
    #[cfg(feature = "profiling")]
    pub(crate) source_lease_count: usize,
    #[cfg(feature = "profiling")]
    pub(crate) unique_source_owner_count: usize,
    #[cfg(feature = "profiling")]
    pub(crate) leased_source_bytes: usize,
    #[cfg(feature = "profiling")]
    pub(crate) unique_source_owner_bytes: usize,
}

#[cfg(any(test, feature = "profiling"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextSourceOwnershipProfile {
    lease_count: usize,
    unique_owner_count: usize,
    leased_bytes: usize,
    unique_owner_bytes: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct PendingShapeJob {
    key: ShapedRunCacheKey,
    request: TextShapeParagraph,
    outcome: Option<TextShapingOutcome<GenerationTaggedShapedRun>>,
}

impl TextShapeParagraph {
    pub(crate) fn horizontal(
        text: impl Into<Arc<str>>,
        style: TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
    ) -> Self {
        Self::horizontal_with_kerning(text, style, base_direction, source_range, true)
    }

    pub(crate) fn horizontal_with_kerning(
        text: impl Into<Arc<str>>,
        style: TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Self {
        Self {
            text: text.into(),
            style,
            base_direction,
            source_range,
            include_kerning,
            vertical_mode: None,
        }
    }

    /// Mirrors a single text span requested by the layout engine. Rich spans use a local source
    /// range; vertical spans retain their shaping mode so they populate the same cache key.
    pub(crate) fn layout_span(
        text: impl Into<Arc<str>>,
        style: TextStyle,
        base_direction: TextDirection,
        vertical_mode: Option<VerticalMode>,
    ) -> Self {
        let text = text.into();
        Self {
            source_range: TextRange {
                start: 0,
                end: text.len(),
            },
            text,
            style,
            base_direction,
            include_kerning: true,
            vertical_mode,
        }
    }

    /// Splits a document into independently cacheable physical paragraphs. The source ranges
    /// remain absolute so later layout and text-input consumers can reuse the same shaped runs.
    pub(crate) fn horizontal_paragraphs(
        text: &str,
        style: TextStyle,
        base_direction: TextDirection,
        document_source_range: TextRange,
    ) -> Vec<Self> {
        if document_source_range
            .end
            .checked_sub(document_source_range.start)
            != Some(text.len())
        {
            return vec![Self::horizontal(
                text,
                style,
                base_direction,
                document_source_range,
            )];
        }

        crate::text::hard_lines(text)
            .into_iter()
            .map(|line| {
                let line_source_range = line.source_range();
                let paragraph = &text[line_source_range.clone()];
                Self::horizontal(
                    paragraph,
                    style.clone(),
                    base_direction,
                    TextRange {
                        start: document_source_range.start + line_source_range.start,
                        end: document_source_range.start + line_source_range.end,
                    },
                )
            })
            .collect()
    }

    fn request(&self) -> BackendShapeRequest<'_> {
        let request = match self.vertical_mode {
            Some(vertical_mode) => BackendShapeRequest::vertical_with_kerning(
                self.text.as_ref(),
                &self.style,
                self.base_direction,
                self.source_range,
                vertical_mode,
                self.include_kerning,
            ),
            None => BackendShapeRequest::horizontal_with_kerning(
                self.text.as_ref(),
                &self.style,
                self.base_direction,
                self.source_range,
                self.include_kerning,
            ),
        };
        request
            .with_language(
                self.style
                    .language
                    .as_deref()
                    .map(str::trim)
                    .filter(|language| !language.is_empty()),
            )
            .with_source_owner(&self.text)
    }

    fn text(&self) -> &str {
        self.text.as_ref()
    }
}

impl TextParallelShapeBatchReport {
    fn for_requests(requested_count: usize, chunk_size: usize, worker_parallelism: usize) -> Self {
        Self {
            requested_count,
            chunk_size: chunk_size.max(1),
            worker_parallelism,
            ..Self::default()
        }
    }
}

impl PendingShapeJob {
    fn new(key: ShapedRunCacheKey, request: TextShapeParagraph) -> Self {
        Self {
            key,
            request,
            outcome: None,
        }
    }

    fn matches_lookup(&self, lookup: &ShapedRunCacheLookupKey<'_>, text: &str) -> bool {
        self.key.matches_lookup(lookup) && self.request.text() == text
    }

    fn shape(&mut self, font_collection: &Arc<FontCollectionService>) {
        let request = self.request.request();
        self.outcome = Some(shape_request_with_generation_outcome_in_font_collection(
            request,
            font_collection,
        ));
    }
}

#[cfg(test)]
pub(crate) fn shape_paragraphs_with_cache(
    pool: &TaskPool,
    cache: &mut ShapedRunCache,
    requests: &[TextShapeParagraph],
    chunk_size: usize,
    work_budget: TextShapingWorkBudget,
) -> TextParallelShapeBatchReport {
    let font_collection = shared_font_collection_service();
    shape_paragraphs_with_cache_in_font_collection(
        pool,
        cache,
        requests,
        chunk_size,
        work_budget,
        &font_collection,
    )
}

pub(crate) fn shape_paragraphs_with_cache_in_font_collection(
    pool: &TaskPool,
    cache: &mut ShapedRunCache,
    requests: &[TextShapeParagraph],
    chunk_size: usize,
    work_budget: TextShapingWorkBudget,
    font_collection: &Arc<FontCollectionService>,
) -> TextParallelShapeBatchReport {
    let report = {
        crate::profile_scope!("runtime", "text.shape_batch", "shape_paragraphs_with_cache");
        let chunk_size = chunk_size.max(1);
        let mut report = TextParallelShapeBatchReport::for_requests(
            requests.len(),
            chunk_size,
            pool.parallelism(),
        );
        #[cfg(feature = "profiling")]
        {
            let source_ownership = source_ownership_profile(requests);
            report.source_lease_count = source_ownership.lease_count;
            report.unique_source_owner_count = source_ownership.unique_owner_count;
            report.leased_source_bytes = source_ownership.leased_bytes;
            report.unique_source_owner_bytes = source_ownership.unique_owner_bytes;
        }
        let mut pending: Vec<PendingShapeJob> = Vec::new();
        let mut pending_by_lookup_fingerprint: HashMap<EphemeralCacheHash, Vec<usize>> =
            HashMap::new();

        let lookup_generation = font_collection.generation();
        let lookup_collection = font_collection.collection_id();
        for request in requests {
            let borrowed = request.request();
            if !borrowed.style.font_size.is_finite() || borrowed.style.font_size <= 0.0 {
                report.invalid_request_count = report.invalid_request_count.saturating_add(1);
                continue;
            }
            let lookup = ShapedRunCacheLookupKey::from_request_in_font_collection(
                &borrowed,
                lookup_collection,
                lookup_generation,
            );
            let pending_lookup_fingerprint = if pending.is_empty() {
                None
            } else {
                Some(lookup.exact_fingerprint())
            };
            if let Some(pending_index) = pending_lookup_fingerprint
                .and_then(|fingerprint| pending_by_lookup_fingerprint.get(&fingerprint))
                .and_then(|candidate_indices| {
                    candidate_indices.iter().copied().find(|&pending_index| {
                        report.pending_lookup_candidate_count =
                            report.pending_lookup_candidate_count.saturating_add(1);
                        pending[pending_index].matches_lookup(&lookup, request.text())
                    })
                })
            {
                report.batch_duplicate_count = report.batch_duplicate_count.saturating_add(1);
                continue;
            }

            if cache.get_with_lookup(&lookup, request.text()).is_some() {
                report.cache_hit_count = report.cache_hit_count.saturating_add(1);
            } else {
                let lookup_fingerprint = match pending_lookup_fingerprint {
                    Some(fingerprint) => fingerprint,
                    None => lookup.exact_fingerprint(),
                };
                let key = cache.own_lookup_key(&lookup);
                let pending_index = pending.len();
                pending.push(PendingShapeJob::new(key, request.clone()));
                report
                    .shaping_work
                    .record_synchronous_request(work_budget, request.text().len());
                pending_by_lookup_fingerprint
                    .entry(lookup_fingerprint)
                    .or_default()
                    .push(pending_index);
                report.cache_miss_count = report.cache_miss_count.saturating_add(1);
            }
        }

        report.shaped_count = pending.len();
        if pending.len() < TEXT_SHAPE_PARALLEL_MIN_JOBS || pool.parallelism() == 1 {
            if !pending.is_empty() {
                report.inline_batch_count = 1;
            }
            for job in &mut pending {
                job.shape(font_collection);
            }
        } else {
            report.parallel_join_count = 1;
            let wait_started = Instant::now();
            parallel_for(pool, pending.as_mut_slice(), chunk_size, |jobs| {
                for job in jobs {
                    job.shape(font_collection);
                }
            });
            report.caller_wait_nanos =
                wait_started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        }

        for job in pending {
            finish_pending_shape_job_in_font_collection(cache, &mut report, job, font_collection);
        }

        report
    };
    #[cfg(feature = "profiling")]
    record_shape_batch_profile(&report);
    report
}

#[cfg(feature = "profiling")]
fn record_shape_batch_profile(report: &TextParallelShapeBatchReport) {
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.requested",
        report.requested_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.cache_hits",
        report.cache_hit_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.cache_misses",
        report.cache_miss_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.batch_duplicates",
        report.batch_duplicate_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.lookup_candidates",
        report.pending_lookup_candidate_count
    );
    crate::profile_counter!("runtime", "text.shape_batch.shaped", report.shaped_count);
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.inserted",
        report.inserted_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.invalid_requests",
        report.invalid_request_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.generation_deferred",
        report.generation_deferred_count
    );
    crate::profile_counter!("runtime", "text.shape_batch.failed", report.failed_count);
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.inline_batches",
        report.inline_batch_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.parallel_joins",
        report.parallel_join_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.caller_wait_nanos",
        report.caller_wait_nanos
    );
    crate::profile_counter!("runtime", "text.shape_batch.chunk_size", report.chunk_size);
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.worker_parallelism",
        report.worker_parallelism
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.work_budget_inline_requests",
        report.shaping_work.inline_request_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.work_budget_oversized_synchronous_requests",
        report.shaping_work.oversized_synchronous_request_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.synchronous_input_bytes",
        report.shaping_work.synchronous_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.max_synchronous_input_bytes",
        report.shaping_work.max_synchronous_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.source_lease_count",
        report.source_lease_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.unique_source_owner_count",
        report.unique_source_owner_count
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.leased_source_bytes",
        report.leased_source_bytes
    );
    crate::profile_counter!(
        "runtime",
        "text.shape_batch.unique_source_owner_bytes",
        report.unique_source_owner_bytes
    );
}

#[cfg(any(test, feature = "profiling"))]
fn source_ownership_profile(requests: &[TextShapeParagraph]) -> TextSourceOwnershipProfile {
    let mut profile = TextSourceOwnershipProfile::default();
    let mut unique_owners = HashSet::with_capacity(requests.len());
    for request in requests {
        profile.lease_count = profile.lease_count.saturating_add(1);
        profile.leased_bytes = profile.leased_bytes.saturating_add(request.text.len());
        let owner = (request.text.as_ptr() as usize, request.text.len());
        if unique_owners.insert(owner) {
            profile.unique_owner_count = profile.unique_owner_count.saturating_add(1);
            profile.unique_owner_bytes = profile
                .unique_owner_bytes
                .saturating_add(request.text.len());
        }
    }
    profile
}

fn finish_pending_shape_job_in_font_collection(
    cache: &mut ShapedRunCache,
    report: &mut TextParallelShapeBatchReport,
    mut job: PendingShapeJob,
    font_collection: &Arc<FontCollectionService>,
) {
    let Some(outcome) = job.outcome.take() else {
        report.failed_count = report.failed_count.saturating_add(1);
        return;
    };
    match outcome {
        TextShapingOutcome::Ready(tagged) => {
            report
                .shaping_diagnostics
                .record_ready_run(&tagged.run, tagged.request_diagnostics);
            if job.key.font_database_generation() == tagged.font_generation
                && tagged.font_generation == font_collection.generation()
            {
                report.inserted_count = report.inserted_count.saturating_add(1);
                cache.insert_ready(job.key, tagged.run);
            } else {
                let failure = TextShapingFailure::font_generation_changed();
                report.shaping_diagnostics.record_deferred_failure(&failure);
                report.generation_deferred_count =
                    report.generation_deferred_count.saturating_add(1);
            }
        }
        TextShapingOutcome::Deferred(failure) => {
            report.shaping_diagnostics.record_deferred_failure(&failure);
            report.generation_deferred_count = report.generation_deferred_count.saturating_add(1);
        }
        TextShapingOutcome::Failed(failure) => {
            report.shaping_diagnostics.record_terminal_failure(&failure);
            report.failed_count = report.failed_count.saturating_add(1);
        }
    }
}

#[cfg(test)]
#[path = "shape_pool/tests.rs"]
mod tests;
