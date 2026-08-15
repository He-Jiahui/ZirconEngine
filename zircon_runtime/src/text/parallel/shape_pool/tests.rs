use std::sync::Arc;

use super::{
    finish_pending_shape_job, shape_paragraphs_with_cache, PendingShapeJob,
    TextParallelShapeBatchReport, TextShapeParagraph,
};
use crate::core::framework::text::TextDirection;
#[cfg(feature = "profiling")]
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};
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
    assert_eq!(
        second.runs[0].source_text.as_ref(),
        "folder-open-outline.svg"
    );
    assert_eq!(second.runs[1].source_text.as_ref(), "workbench_panel.rs");
    assert_eq!(
        second.runs[2].source_text.as_ref(),
        "retained_text_metrics.rs"
    );
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

#[cfg(feature = "profiling")]
#[test]
fn shape_batch_profiles_fixed_stage_and_cache_counters() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "text-shape-batch-profile".to_string();
    config.max_spans = 4;
    config.max_counters = 32;
    start_capture(config);

    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let mut cache = ShapedRunCache::with_capacity(16);
    let requests = paragraphs(&["same line", "same line"]);
    let batch = shape_paragraphs_with_cache(&pool, &mut cache, &requests, 1);

    let profile = snapshot();
    reset_capture();
    assert_eq!(batch.report.requested_count, 2);
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
