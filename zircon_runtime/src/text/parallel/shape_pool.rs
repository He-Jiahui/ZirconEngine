//! Parallel paragraph shaping batch helpers.

use std::{collections::HashMap, sync::Arc, time::Instant};

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::core::runtime::tasks::{parallel_for, TaskPool};
use crate::text::cache::{ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheLookupKey};
use crate::text::font::shared_font_database_generation;
use crate::text::layout_session::{
    shape_fallback_for_error, try_shape_request_through_canonical_service,
};
use crate::text::{BackendShapeRequest, ShapedGlyphRun};
use crate::text::{TextRange, TextStyle};

const TEXT_SHAPE_PARALLEL_MIN_JOBS: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapeParagraph {
    text: Arc<str>,
    style: TextStyle,
    base_direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
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
    pub(crate) inline_batch_count: usize,
    pub(crate) parallel_join_count: usize,
    pub(crate) caller_wait_nanos: u64,
    pub(crate) chunk_size: usize,
    pub(crate) worker_parallelism: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextParallelShapeBatch {
    pub(crate) runs: Vec<Arc<ShapedGlyphRun>>,
    pub(crate) report: TextParallelShapeBatchReport,
}

#[derive(Clone, Debug, PartialEq)]
struct PendingShapeJob {
    key: ShapedRunCacheKey,
    request: TextShapeParagraph,
    output_indices: Vec<usize>,
    run: Option<ShapedGlyphRun>,
    cacheable: bool,
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
        BackendShapeRequest::horizontal_with_kerning(
            self.text.as_ref(),
            &self.style,
            self.base_direction,
            self.source_range,
            self.include_kerning,
        )
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
    fn new(key: ShapedRunCacheKey, request: TextShapeParagraph, output_index: usize) -> Self {
        Self {
            key,
            request,
            output_indices: vec![output_index],
            run: None,
            cacheable: true,
        }
    }

    fn matches_lookup(&self, lookup: &ShapedRunCacheLookupKey<'_>, text: &str) -> bool {
        self.key.matches_lookup(lookup) && self.request.text() == text
    }

    fn shape(&mut self) {
        let request = self.request.request();
        match try_shape_request_through_canonical_service(request) {
            Ok(run) => {
                self.cacheable =
                    self.key.font_database_generation() == shared_font_database_generation();
                self.run = Some(run);
            }
            Err(error) => {
                self.cacheable = !matches!(&error, TextLayoutError::FontGenerationChanged);
                self.run = Some(shape_fallback_for_error(request, &error));
            }
        }
    }
}

pub(crate) fn shape_paragraphs_with_cache(
    pool: &TaskPool,
    cache: &mut ShapedRunCache,
    requests: &[TextShapeParagraph],
    chunk_size: usize,
) -> TextParallelShapeBatch {
    let chunk_size = chunk_size.max(1);
    let mut report =
        TextParallelShapeBatchReport::for_requests(requests.len(), chunk_size, pool.parallelism());
    let mut runs = vec![None; requests.len()];
    let mut pending: Vec<PendingShapeJob> = Vec::new();
    let mut pending_by_lookup_fingerprint: HashMap<u64, Vec<usize>> = HashMap::new();

    for (index, request) in requests.iter().enumerate() {
        let borrowed = request.request();
        if !borrowed.style.font_size.is_finite() || borrowed.style.font_size <= 0.0 {
            runs[index] = Some(Arc::new(shape_fallback_for_error(
                borrowed,
                &TextLayoutError::InvalidFontSize,
            )));
            report.invalid_request_count = report.invalid_request_count.saturating_add(1);
            continue;
        }
        let lookup = ShapedRunCacheLookupKey::from_request(&borrowed);
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
            pending[pending_index].output_indices.push(index);
            report.batch_duplicate_count = report.batch_duplicate_count.saturating_add(1);
            continue;
        }

        if let Some(run) = cache.get_with_lookup(&lookup, request.text()) {
            runs[index] = Some(run);
            report.cache_hit_count = report.cache_hit_count.saturating_add(1);
        } else {
            let lookup_fingerprint = match pending_lookup_fingerprint {
                Some(fingerprint) => fingerprint,
                None => lookup.exact_fingerprint(),
            };
            let key = cache.own_lookup_key(&lookup);
            let pending_index = pending.len();
            pending.push(PendingShapeJob::new(key, request.clone(), index));
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
            job.shape();
        }
    } else {
        report.parallel_join_count = 1;
        let wait_started = Instant::now();
        parallel_for(pool, pending.as_mut_slice(), chunk_size, |jobs| {
            for job in jobs {
                job.shape();
            }
        });
        report.caller_wait_nanos = wait_started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    }

    for job in pending {
        finish_pending_shape_job(cache, &mut report, &mut runs, job);
    }

    let ordered_runs = runs.into_iter().flatten().collect::<Vec<_>>();
    debug_assert_eq!(ordered_runs.len(), requests.len());

    TextParallelShapeBatch {
        runs: ordered_runs,
        report,
    }
}

fn finish_pending_shape_job(
    cache: &mut ShapedRunCache,
    report: &mut TextParallelShapeBatchReport,
    runs: &mut [Option<Arc<ShapedGlyphRun>>],
    mut job: PendingShapeJob,
) {
    let Some(run) = job.run.take() else {
        return;
    };
    let run = if job.cacheable {
        report.inserted_count = report.inserted_count.saturating_add(1);
        cache.insert(job.key, run)
    } else {
        report.generation_deferred_count = report.generation_deferred_count.saturating_add(1);
        Arc::new(run)
    };
    for output_index in job.output_indices {
        runs[output_index] = Some(Arc::clone(&run));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        finish_pending_shape_job, shape_paragraphs_with_cache, PendingShapeJob,
        TextParallelShapeBatchReport, TextShapeParagraph,
    };
    use crate::core::framework::text::TextDirection;
    use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
    use crate::text::cache::ShapedRunCache;
    use crate::text::layout_session::shape_request_through_canonical_service;
    use crate::text::{TextRange, TextStyle};

    #[test]
    fn render_perf_text_parallel_shape_count() {
        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let mut cache = ShapedRunCache::with_capacity(16);
        let first_requests = paragraphs(&[
            "editor base.zui",
            "folder-open-outline.svg",
            "workbench_panel.rs",
        ]);

        cache.begin_frame(1);
        let first = shape_paragraphs_with_cache(&pool, &mut cache, &first_requests, 1);
        let first_cache_report = cache.report();

        assert_eq!(first.runs.len(), first_requests.len());
        assert_eq!(first.report.cache_hit_count, 0);
        assert_eq!(first.report.cache_miss_count, first_requests.len());
        assert_eq!(first.report.shaped_count, first_requests.len());
        assert_eq!(first.report.inserted_count, first_requests.len());
        assert_eq!(first_cache_report.miss_count, first_requests.len() as u64);
        assert_eq!(first_cache_report.insert_count, first_requests.len() as u64);

        cache.begin_frame(2);
        let second_requests = vec![
            paragraph("folder-open-outline.svg", &style),
            paragraph("workbench_panel.rs", &style),
            paragraph("retained_text_metrics.rs", &style),
        ];
        let second = shape_paragraphs_with_cache(&pool, &mut cache, &second_requests, 1);
        let second_cache_report = cache.report();

        assert_eq!(second.runs.len(), second_requests.len());
        assert_eq!(second.report.cache_hit_count, 2);
        assert_eq!(second.report.cache_miss_count, 1);
        assert_eq!(second.report.shaped_count, 1);
        assert_eq!(second.report.inserted_count, 1);
        assert_eq!(
            second_cache_report.hit_count, 2,
            "cached paragraphs should not enter the parallel shape work set"
        );
        assert_eq!(
            second_cache_report.miss_count, 1,
            "only the newly visible paragraph should miss the shaped-run cache"
        );
        assert_eq!(second_cache_report.insert_count, 1);
        assert_eq!(second.runs[0].source_text, "folder-open-outline.svg");
        assert_eq!(second.runs[1].source_text, "workbench_panel.rs");
        assert_eq!(second.runs[2].source_text, "retained_text_metrics.rs");
    }

    #[test]
    fn text_parallel_shape_batch_deduplicates_same_frame_misses() {
        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let mut cache = ShapedRunCache::with_capacity(16);
        let requests = vec![
            paragraph("editor base.zui", &style),
            paragraph("folder-open-outline.svg", &style),
            paragraph("editor base.zui", &style),
            paragraph("folder-open-outline.svg", &style),
        ];

        cache.begin_frame(1);
        let batch = shape_paragraphs_with_cache(&pool, &mut cache, &requests, 1);
        let cache_report = cache.report();

        assert_eq!(batch.runs.len(), requests.len());
        assert_eq!(batch.report.cache_miss_count, 2);
        assert_eq!(batch.report.batch_duplicate_count, 2);
        assert_eq!(batch.report.pending_lookup_candidate_count, 2);
        assert_eq!(batch.report.shaped_count, 2);
        assert_eq!(batch.report.inserted_count, 2);
        assert_eq!(cache_report.miss_count, 2);
        assert_eq!(cache_report.insert_count, 2);
        assert!(Arc::ptr_eq(&batch.runs[0], &batch.runs[2]));
        assert!(Arc::ptr_eq(&batch.runs[1], &batch.runs[3]));
    }

    #[test]
    fn text_paragraph_dirty_reshapes_edited_only() {
        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let mut cache = ShapedRunCache::with_capacity(16);
        let original = "one\ntwo\nthree";
        let edited = "one\nTWO\nthree";

        cache.begin_frame(1);
        let original_requests = TextShapeParagraph::horizontal_paragraphs(
            original,
            style.clone(),
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: original.len(),
            },
        );
        let original_batch = shape_paragraphs_with_cache(&pool, &mut cache, &original_requests, 1);

        cache.begin_frame(2);
        let edited_requests = TextShapeParagraph::horizontal_paragraphs(
            edited,
            style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: edited.len(),
            },
        );
        let edited_batch = shape_paragraphs_with_cache(&pool, &mut cache, &edited_requests, 1);

        assert_eq!(original_requests.len(), 3);
        assert_eq!(original_batch.report.shaped_count, 3);
        assert_eq!(edited_requests.len(), 3);
        assert_eq!(edited_batch.report.cache_hit_count, 2);
        assert_eq!(edited_batch.report.cache_miss_count, 1);
        assert_eq!(edited_batch.report.shaped_count, 1);
        assert_eq!(
            edited_batch
                .runs
                .iter()
                .map(|run| run.source_text.as_ref())
                .collect::<Vec<_>>(),
            vec!["one\n", "TWO\n", "three"]
        );
    }

    #[test]
    fn text_paragraphs_preserve_absolute_source_ranges() {
        let text = "one\ntwo";
        let paragraphs = TextShapeParagraph::horizontal_paragraphs(
            text,
            compact_editor_label_style(),
            TextDirection::LeftToRight,
            TextRange {
                start: 40,
                end: 40 + text.len(),
            },
        );

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].source_range, TextRange { start: 40, end: 44 });
        assert_eq!(paragraphs[1].source_range, TextRange { start: 44, end: 47 });
        assert_eq!(paragraphs[0].text(), "one\n");
        assert_eq!(paragraphs[1].text(), "two");
    }

    #[test]
    fn parallel_shape_run_reuses_the_request_source_allocation() {
        let source: Arc<str> = Arc::from("one shared paragraph source");
        let paragraph = TextShapeParagraph::horizontal(
            Arc::clone(&source),
            compact_editor_label_style(),
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: source.len(),
            },
        );

        let run = shape_request_through_canonical_service(paragraph.request());

        assert!(Arc::ptr_eq(&source, &run.source_text));
    }

    #[test]
    fn text_parallel_shape_batch_indexes_unique_pending_misses() {
        const UNIQUE_REQUEST_COUNT: usize = 32;

        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let mut cache = ShapedRunCache::with_capacity(UNIQUE_REQUEST_COUNT);
        let mut requests = Vec::with_capacity(UNIQUE_REQUEST_COUNT);
        for index in 0..UNIQUE_REQUEST_COUNT {
            let text = format!("visible-row-{index}.zr");
            requests.push(paragraph(text.as_str(), &style));
        }

        cache.begin_frame(1);
        let batch = shape_paragraphs_with_cache(&pool, &mut cache, &requests, 4);

        assert_eq!(batch.runs.len(), UNIQUE_REQUEST_COUNT);
        assert_eq!(batch.report.cache_miss_count, UNIQUE_REQUEST_COUNT);
        assert_eq!(batch.report.shaped_count, UNIQUE_REQUEST_COUNT);
        assert!(
            batch.report.pending_lookup_candidate_count <= UNIQUE_REQUEST_COUNT,
            "unique misses must only compare same-fingerprint candidates"
        );
        assert_eq!(batch.report.parallel_join_count, 1);
        assert_eq!(batch.report.inline_batch_count, 0);
    }

    #[test]
    fn small_shape_batches_stay_inline_without_a_pool_join() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
        let mut cache = ShapedRunCache::with_capacity(16);
        let requests = paragraphs(&["one", "two", "three"]);

        cache.begin_frame(1);
        let batch = shape_paragraphs_with_cache(&pool, &mut cache, &requests, 1);

        assert_eq!(batch.runs.len(), requests.len());
        assert_eq!(batch.report.inline_batch_count, 1);
        assert_eq!(batch.report.parallel_join_count, 0);
        assert_eq!(batch.report.caller_wait_nanos, 0);
        assert_eq!(batch.report.worker_parallelism, 4);
        assert_eq!(batch.runs[0].source_text.as_ref(), "one");
    }

    #[test]
    fn generation_deferred_shape_is_returned_without_caching() {
        let style = compact_editor_label_style();
        let request = paragraph("retry next frame", &style);
        let mut cache = ShapedRunCache::with_capacity(16);
        let borrowed = request.request();
        let key = cache.own_lookup_key(&crate::text::cache::ShapedRunCacheLookupKey::from_request(
            &borrowed,
        ));
        let mut job = PendingShapeJob::new(key, request.clone(), 0);
        job.run = Some(shape_request_through_canonical_service(request.request()));
        job.cacheable = false;
        let mut report = TextParallelShapeBatchReport::for_requests(1, 1, 1);
        let mut runs = vec![None];

        finish_pending_shape_job(&mut cache, &mut report, &mut runs, job);

        assert_eq!(report.inserted_count, 0);
        assert_eq!(report.generation_deferred_count, 1);
        assert_eq!(cache.report().insert_count, 0);
        assert_eq!(
            runs[0].as_deref().map(|run| run.source_text.as_ref()),
            Some("retry next frame")
        );
    }

    #[test]
    fn invalid_prewarm_does_not_poison_the_valid_one_pixel_cache_key() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let mut cache = ShapedRunCache::with_capacity(16);
        let mut invalid_style = compact_editor_label_style();
        invalid_style.font_size = 0.0;
        let invalid = paragraph("one pixel", &invalid_style);

        let invalid_batch = shape_paragraphs_with_cache(&pool, &mut cache, &[invalid], 1);

        assert_eq!(invalid_batch.report.invalid_request_count, 1);
        assert_eq!(invalid_batch.report.inserted_count, 0);
        assert_eq!(cache.report().insert_count, 0);

        let mut valid_style = invalid_style;
        valid_style.font_size = 1.0;
        let valid = paragraph("one pixel", &valid_style);
        let valid_batch = shape_paragraphs_with_cache(&pool, &mut cache, &[valid], 1);

        assert_eq!(valid_batch.report.invalid_request_count, 0);
        assert_eq!(valid_batch.report.cache_hit_count, 0);
        assert_eq!(valid_batch.report.cache_miss_count, 1);
        assert_eq!(valid_batch.report.inserted_count, 1);
    }

    fn paragraphs(texts: &[&str]) -> Vec<TextShapeParagraph> {
        let style = compact_editor_label_style();
        texts.iter().map(|text| paragraph(text, &style)).collect()
    }

    fn paragraph(text: &str, style: &TextStyle) -> TextShapeParagraph {
        TextShapeParagraph::horizontal(
            Arc::<str>::from(text),
            style.clone(),
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        )
    }

    fn compact_editor_label_style() -> TextStyle {
        TextStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..TextStyle::default()
        }
    }
}
