use std::sync::Arc;

use super::{
    PendingShapeJob, TextParallelShapeBatchReport, TextShapeParagraph, finish_pending_shape_job,
    shape_paragraphs_with_cache, source_ownership_profile,
};

use crate::core::framework::text::{TextDirection, TextLayoutError};
#[cfg(feature = "profiling")]
use crate::core::runtime::diagnostics::profiling::{
    ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
};
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use crate::text::cache::ShapedRunCache;
use crate::text::layout_session::{GenerationTaggedShapedRun, shape_request_outcome};
use crate::text::shaping::{TextShapingOutcome, TextShapingWorkBudget};
use crate::text::{TextRange, TextStyle};

#[test]
fn source_ownership_profile_distinguishes_leases_from_unique_arc_owners() {
    let source: Arc<str> = Arc::from("shared paragraph");
    let style = TextStyle::default();
    let requests = vec![
        TextShapeParagraph::horizontal(
            Arc::clone(&source),
            style.clone(),
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: source.len(),
            },
        ),
        TextShapeParagraph::horizontal(
            Arc::clone(&source),
            style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: source.len(),
            },
        ),
    ];

    let profile = source_ownership_profile(&requests);

    assert_eq!(profile.lease_count, 2);
    assert_eq!(profile.unique_owner_count, 1);
    assert_eq!(profile.leased_bytes, source.len() * 2);
    assert_eq!(profile.unique_owner_bytes, source.len());
}

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
    let first = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &first_requests,
        1,
        TextShapingWorkBudget::default(),
    );
    let first_cache_report = cache.report();

    assert_eq!(first.cache_hit_count, 0);
    assert_eq!(first.cache_miss_count, first_requests.len());
    assert_eq!(first.shaped_count, first_requests.len());
    assert_eq!(first.inserted_count, first_requests.len());
    assert_eq!(
        first.shaping_diagnostics.backend_routes.direct_run_count,
        first_requests.len() as u64
    );
    assert_eq!(
        first.shaping_work.inline_request_count,
        first_requests.len()
    );
    assert_eq!(first.shaping_work.oversized_synchronous_request_count, 0);
    assert_eq!(
        first.shaping_work.synchronous_input_bytes,
        first_requests
            .iter()
            .map(|request| request.text().len())
            .sum::<usize>()
    );
    assert_eq!(first_cache_report.miss_count, first_requests.len() as u64);
    assert_eq!(first_cache_report.insert_count, first_requests.len() as u64);

    cache.begin_frame(2);
    let second_requests = vec![
        paragraph("folder-open-outline.svg", &style),
        paragraph("workbench_panel.rs", &style),
        paragraph("retained_text_metrics.rs", &style),
    ];
    let second = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &second_requests,
        1,
        TextShapingWorkBudget::default(),
    );
    let second_cache_report = cache.report();

    assert_eq!(second.cache_hit_count, 2);
    assert_eq!(second.cache_miss_count, 1);
    assert_eq!(second.shaped_count, 1);
    assert_eq!(second.inserted_count, 1);
    assert_eq!(
        second.shaping_diagnostics.backend_routes.direct_run_count,
        1
    );
    assert_eq!(second.shaping_work.inline_request_count, 1);
    assert_eq!(
        second.shaping_work.synchronous_input_bytes,
        "retained_text_metrics.rs".len()
    );
    assert_eq!(
        second_cache_report.hit_count, 2,
        "cached paragraphs should not enter the parallel shape work set"
    );
    assert_eq!(
        second_cache_report.miss_count, 1,
        "only the newly visible paragraph should miss the shaped-run cache"
    );
    assert_eq!(second_cache_report.insert_count, 1);
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
    let batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &requests,
        1,
        TextShapingWorkBudget::default(),
    );
    let cache_report = cache.report();

    assert_eq!(batch.cache_miss_count, 2);
    assert_eq!(batch.batch_duplicate_count, 2);
    assert_eq!(batch.pending_lookup_candidate_count, 2);
    assert_eq!(batch.shaped_count, 2);
    assert_eq!(batch.inserted_count, 2);
    assert_eq!(batch.shaping_work.inline_request_count, 2);
    assert_eq!(batch.shaping_work.oversized_synchronous_request_count, 0);
    assert_eq!(cache_report.miss_count, 2);
    assert_eq!(cache_report.insert_count, 2);
}

#[test]
fn text_parallel_shape_batch_reports_oversized_work_without_splitting_the_request() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let mut cache = ShapedRunCache::with_capacity(4);
    let request = paragraph("complete", &compact_editor_label_style());
    let budget = TextShapingWorkBudget::new(4).expect("non-zero budget");

    cache.begin_frame(1);
    let batch = shape_paragraphs_with_cache(&pool, &mut cache, &[request], 1, budget);

    assert_eq!(batch.shaped_count, 1);
    assert_eq!(batch.inserted_count, 1);
    assert_eq!(batch.shaping_work.inline_request_count, 0);
    assert_eq!(batch.shaping_work.oversized_synchronous_request_count, 1);
    assert_eq!(batch.shaping_work.synchronous_input_bytes, "complete".len());
    assert_eq!(
        batch.shaping_work.max_synchronous_input_bytes,
        "complete".len()
    );
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
    let original_batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &original_requests,
        1,
        TextShapingWorkBudget::default(),
    );

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
    let edited_batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &edited_requests,
        1,
        TextShapingWorkBudget::default(),
    );

    assert_eq!(original_requests.len(), 3);
    assert_eq!(original_batch.shaped_count, 3);
    assert_eq!(edited_requests.len(), 3);
    assert_eq!(edited_batch.cache_hit_count, 2);
    assert_eq!(edited_batch.cache_miss_count, 1);
    assert_eq!(edited_batch.shaped_count, 1);
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

    let outcome = shape_request_outcome(paragraph.request());
    let TextShapingOutcome::Ready(run) = outcome else {
        panic!("a valid paragraph must shape without an error outcome");
    };

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
    let batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &requests,
        4,
        TextShapingWorkBudget::default(),
    );

    assert_eq!(batch.cache_miss_count, UNIQUE_REQUEST_COUNT);
    assert_eq!(batch.shaped_count, UNIQUE_REQUEST_COUNT);
    assert!(
        batch.pending_lookup_candidate_count <= UNIQUE_REQUEST_COUNT,
        "unique misses must only compare same-fingerprint candidates"
    );
    assert_eq!(batch.parallel_join_count, 1);
    assert_eq!(batch.inline_batch_count, 0);
}

#[test]
fn small_shape_batches_stay_inline_without_a_pool_join() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let mut cache = ShapedRunCache::with_capacity(16);
    let requests = paragraphs(&["one", "two", "three"]);

    cache.begin_frame(1);
    let batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &requests,
        1,
        TextShapingWorkBudget::default(),
    );

    assert_eq!(batch.inline_batch_count, 1);
    assert_eq!(batch.parallel_join_count, 0);
    assert_eq!(batch.caller_wait_nanos, 0);
    assert_eq!(batch.worker_parallelism, 4);
}

#[cfg(feature = "profiling")]
#[test]
fn shape_batch_profiles_fixed_stage_and_cache_counters() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "text-shape-batch-profile".to_string();
    config.max_spans = 4;
    config.max_counters = 64;
    start_capture(config);

    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let mut cache = ShapedRunCache::with_capacity(16);
    let requests = paragraphs(&["same line", "same line"]);
    let batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &requests,
        1,
        TextShapingWorkBudget::default(),
    );

    let profile = snapshot();
    reset_capture();
    assert_eq!(batch.requested_count, 2);
    let shape_span = profile
        .spans
        .iter()
        .find(|span| {
            span.stream == "runtime"
                && span.category == "text.shape_batch"
                && span.name == "shape_paragraphs_with_cache"
        })
        .unwrap_or_else(|| panic!("missing text shape batch span"));
    let requested_counter = profile
        .counters
        .iter()
        .find(|counter| counter.stream == "runtime" && counter.name == "text.shape_batch.requested")
        .unwrap_or_else(|| panic!("missing requested shape counter"));
    assert!(
        requested_counter.timestamp_us
            >= shape_span.start_us.saturating_add(shape_span.duration_us),
        "counters must be recorded after the measured shape scope closes"
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.requested"),
        2.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.cache_misses"),
        1.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.batch_duplicates"),
        1.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.shaped"),
        1.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.source_lease_count"),
        2.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.unique_source_owner_count"),
        2.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.leased_source_bytes"),
        ("same line".len() * 2) as f64
    );
    assert_eq!(
        profile_counter_value(&profile, "text.shape_batch.unique_source_owner_bytes"),
        ("same line".len() * 2) as f64
    );
    assert_eq!(
        profile_counter_value(&profile, "text_shape_source_materialization_count"),
        1.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text_shape_source_owner_reuse_count"),
        1.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text_shape_source_allocation_count"),
        0.0
    );
    assert_eq!(
        profile_counter_value(&profile, "text_shape_source_allocation_byte_count"),
        0.0
    );
}

#[test]
fn generation_deferred_shape_is_not_published_or_cached() {
    let style = compact_editor_label_style();
    let request = paragraph("retry next frame", &style);
    let mut cache = ShapedRunCache::with_capacity(16);
    let borrowed = request.request();
    let key = cache.own_lookup_key(&crate::text::cache::ShapedRunCacheLookupKey::from_request(
        &borrowed,
    ));
    let mut job = PendingShapeJob::new(key, request.clone());
    job.outcome = Some(TextShapingOutcome::deferred(
        TextLayoutError::FontGenerationChanged,
    ));
    let mut report = TextParallelShapeBatchReport::for_requests(1, 1, 1);
    finish_pending_shape_job(&mut cache, &mut report, job);

    assert_eq!(report.inserted_count, 0);
    assert_eq!(report.generation_deferred_count, 1);
    assert_eq!(report.shaping_diagnostics.failures.deferred_count, 1);
    assert_eq!(report.shaping_diagnostics.failures.terminal_count, 0);
    assert_eq!(
        report.shaping_diagnostics.backend_routes.deferred_run_count,
        1
    );
    assert_eq!(
        report.shaping_diagnostics.backend_routes.terminal_run_count,
        0
    );
    assert_eq!(cache.report().insert_count, 0);
}

#[test]
fn ready_worker_result_from_a_retired_generation_is_not_cached() {
    let style = compact_editor_label_style();
    let request = paragraph("retired worker result", &style);
    let mut cache = ShapedRunCache::with_capacity(16);
    let borrowed = request.request();
    let key = cache.own_lookup_key(&crate::text::cache::ShapedRunCacheLookupKey::from_request(
        &borrowed,
    ));
    let run = shape_request_outcome(borrowed)
        .into_result()
        .expect("test input must shape at a stable generation");
    let retired_generation = key.font_database_generation().saturating_sub(1);
    let mut job = PendingShapeJob::new(key, request);
    job.outcome = Some(TextShapingOutcome::Ready(GenerationTaggedShapedRun {
        run,
        font_generation: retired_generation,
        request_diagnostics: Default::default(),
    }));
    let mut report = TextParallelShapeBatchReport::for_requests(1, 1, 1);
    finish_pending_shape_job(&mut cache, &mut report, job);

    assert_eq!(report.inserted_count, 0);
    assert_eq!(report.generation_deferred_count, 1);
    assert_eq!(report.shaping_diagnostics.failures.deferred_count, 1);
    assert_eq!(report.shaping_diagnostics.failures.terminal_count, 0);
    assert_eq!(
        report.shaping_diagnostics.backend_routes.deferred_run_count,
        1
    );
    assert_eq!(cache.report().insert_count, 0);
}

#[test]
fn failed_shape_is_not_published_or_cached() {
    let style = compact_editor_label_style();
    let request = paragraph("retry after repair", &style);
    let mut cache = ShapedRunCache::with_capacity(16);
    let borrowed = request.request();
    let key = cache.own_lookup_key(&crate::text::cache::ShapedRunCacheLookupKey::from_request(
        &borrowed,
    ));
    let mut job = PendingShapeJob::new(key, request);
    job.outcome = Some(TextShapingOutcome::failed(TextLayoutError::BidiInvariant));
    let mut report = TextParallelShapeBatchReport::for_requests(1, 1, 1);
    finish_pending_shape_job(&mut cache, &mut report, job);

    assert_eq!(report.inserted_count, 0);
    assert_eq!(report.failed_count, 1);
    assert_eq!(cache.report().insert_count, 0);
}

#[test]
fn missing_shape_outcome_is_reported_as_failed_without_cache_publication() {
    let style = compact_editor_label_style();
    let request = paragraph("missing worker outcome", &style);
    let mut cache = ShapedRunCache::with_capacity(16);
    let borrowed = request.request();
    let key = cache.own_lookup_key(&crate::text::cache::ShapedRunCacheLookupKey::from_request(
        &borrowed,
    ));
    let job = PendingShapeJob::new(key, request);
    let mut report = TextParallelShapeBatchReport::for_requests(1, 1, 1);

    finish_pending_shape_job(&mut cache, &mut report, job);

    assert_eq!(report.failed_count, 1);
    assert_eq!(report.inserted_count, 0);
    assert_eq!(cache.report().insert_count, 0);
}

#[test]
fn invalid_prewarm_does_not_poison_the_valid_one_pixel_cache_key() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let mut cache = ShapedRunCache::with_capacity(16);
    let mut invalid_style = compact_editor_label_style();
    invalid_style.font_size = 0.0;
    let invalid = paragraph("one pixel", &invalid_style);

    let invalid_batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &[invalid],
        1,
        TextShapingWorkBudget::default(),
    );

    assert_eq!(invalid_batch.invalid_request_count, 1);
    assert_eq!(invalid_batch.inserted_count, 0);
    assert_eq!(cache.report().insert_count, 0);

    let mut valid_style = invalid_style;
    valid_style.font_size = 1.0;
    let valid = paragraph("one pixel", &valid_style);
    let valid_batch = shape_paragraphs_with_cache(
        &pool,
        &mut cache,
        &[valid],
        1,
        TextShapingWorkBudget::default(),
    );

    assert_eq!(valid_batch.invalid_request_count, 0);
    assert_eq!(valid_batch.cache_hit_count, 0);
    assert_eq!(valid_batch.cache_miss_count, 1);
    assert_eq!(valid_batch.inserted_count, 1);
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

#[cfg(feature = "profiling")]
fn profile_counter_value(
    profile: &crate::core::runtime::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    profile
        .counters
        .iter()
        .find(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value)
        .unwrap_or_else(|| panic!("missing profile counter: {name}"))
}
