use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;
use crate::core::math::{UVec2, Vec2};
use crate::core::runtime::tasks::TaskPoolOptions;
use crate::text::raster::{GlyphBitmap, SwashRasterRequest};
use swash::FontRef;

use super::raster_pool::{
    TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC, TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC,
    TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC, TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC,
    TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC, TEXT_RASTER_WORKER_QUEUED_DIAGNOSTIC,
    TEXT_RASTER_WORKER_RUNNING_DIAGNOSTIC, TextRasterCompletionDrainBudget,
    TextRasterThreadBudgetSource, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkerPool, TextRasterWorkerPoolFrameSampler, TextRasterWorkerPoolOptions,
    TextRasterWorkerRequestError,
};

const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));

#[test]
fn text_raster_worker_pool_completes_real_swash_raster_work() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse as face 0");
    let glyph_id = font.charmap().map('P');
    assert_ne!(glyph_id, 0, "test glyph should be present in FiraSans");

    let pool = TextRasterWorkerPool::new(TextRasterWorkerPoolOptions::new(1)).unwrap();
    let face_epoch = 3;
    pool.request(TextRasterWorkItem::new(
        TextRasterWorkId::new(1),
        face_epoch,
        Arc::<[u8]>::from(TEST_FONT_BYTES),
        SwashRasterRequest::alpha_outline(0, glyph_id, 18.0, true),
    ))
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        let mut drain = pool.drain_completed_for_face_epoch(
            face_epoch,
            TextRasterCompletionDrainBudget::new(1, usize::MAX),
        );
        if let Some(result) = drain.accepted.pop() {
            break result;
        }
        assert!(
            Instant::now() < deadline,
            "text raster worker should publish a completion"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    let bitmap = result.result.expect("worker should rasterize alpha glyph");

    assert_eq!(result.id, TextRasterWorkId::new(1));
    assert_eq!(result.face_epoch, face_epoch);
    assert!(bitmap.size.x > 0);
    assert!(bitmap.size.y > 0);
    assert!(bitmap.has_expected_data_len());
    assert!(
        bitmap.data.iter().any(|coverage| *coverage > 0),
        "worker result should contain glyph coverage"
    );

    while pool.diagnostics().in_flight != 0 {
        assert!(
            Instant::now() < deadline,
            "worker should retire the published work item"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completed, 1);
    assert_eq!(diagnostics.failed, 0);
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.queued, 0);
    assert_eq!(diagnostics.running, 0);
    assert_eq!(diagnostics.queue_peak, 1);
}

#[test]
fn text_raster_worker_pool_bounded_queue_rejects_overflow_without_workers() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(0),
    );
    assert!(pool.request_channel_guard_is_alive_for_test());

    let error = pool
        .try_request(TextRasterWorkItem::new(
            TextRasterWorkId::new(10),
            1,
            Arc::<[u8]>::from(TEST_FONT_BYTES),
            SwashRasterRequest::alpha_outline(0, 1, 16.0, true),
        ))
        .expect_err("zero-depth queue without a waiting worker must reject");

    assert_eq!(
        error,
        TextRasterWorkerRequestError::QueueFull(TextRasterWorkId::new(10))
    );
    assert_eq!(pool.diagnostics().in_flight, 0);
    assert_eq!(pool.diagnostics().queue_peak, 0);
}

#[test]
fn text_raster_worker_pool_disconnected_request_channel_fails_without_panicking() {
    let mut pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    pool.disconnect_request_channel_for_test();

    let error = pool
        .request(TextRasterWorkItem::new(
            TextRasterWorkId::new(11),
            1,
            Arc::<[u8]>::from(TEST_FONT_BYTES),
            SwashRasterRequest::alpha_outline(0, 1, 16.0, true),
        ))
        .expect_err("a disconnected raster request channel must fail closed");

    assert!(
        error
            .to_string()
            .contains("text raster worker request channel closed"),
        "unexpected error: {error}"
    );
    assert_eq!(pool.diagnostics().in_flight, 0);
    assert_eq!(pool.diagnostics().queue_peak, 0);
}

#[test]
fn text_raster_worker_pool_cancels_queued_work_without_publishing_a_completion() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let work_id = TextRasterWorkId::new(12);
    pool.request(TextRasterWorkItem::new(
        work_id,
        1,
        Arc::<[u8]>::from(TEST_FONT_BYTES),
        SwashRasterRequest::alpha_outline(0, 1, 16.0, true),
    ))
    .unwrap();

    assert!(pool.cancel(work_id));
    assert!(pool.process_next_request_for_test());
    let drain =
        pool.drain_completed_for_face_epoch(1, TextRasterCompletionDrainBudget::new(1, usize::MAX));
    assert!(drain.accepted.is_empty());

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.cancelled, 1);
    assert_eq!(diagnostics.in_flight, 0);
}

#[test]
fn text_raster_worker_pool_batch_skips_cancelled_work_and_completes_compatible_glyphs() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse as face 0");
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1)
            .with_queue_depth(3)
            .with_completion_queue_depth(3),
    );
    let font_data = Arc::<[u8]>::from(TEST_FONT_BYTES);
    for (work_id, character) in [(40, 'P'), (41, 'r'), (42, 'i')] {
        let glyph_id = font.charmap().map(character);
        assert_ne!(
            glyph_id, 0,
            "test glyph {character:?} should be present in FiraSans"
        );
        pool.request(TextRasterWorkItem::new(
            TextRasterWorkId::new(work_id),
            1,
            Arc::clone(&font_data),
            SwashRasterRequest::alpha_outline(0, glyph_id, 18.0, true).with_font_identity([1, 3]),
        ))
        .expect("compatible glyphs should enter the worker queue");
    }

    assert!(pool.cancel(TextRasterWorkId::new(40)));
    assert_eq!(pool.process_next_batch_for_test(), 3);

    let drain =
        pool.drain_completed_for_face_epoch(1, TextRasterCompletionDrainBudget::new(3, usize::MAX));
    assert_eq!(drain.face_invalidated_count, 0);
    assert_eq!(drain.accepted.len(), 2);
    assert_eq!(drain.accepted[0].id, TextRasterWorkId::new(41));
    assert_eq!(drain.accepted[1].id, TextRasterWorkId::new(42));
    for result in drain.accepted {
        assert!(
            result.result.is_ok(),
            "compatible work should still rasterize"
        );
    }

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.cancelled, 1);
    assert_eq!(diagnostics.completed, 2);
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.running, 0);
}

#[test]
fn text_raster_worker_pool_options_use_async_compute_budget() {
    let options = TextRasterWorkerPoolOptions::from_task_pool_options(
        &TaskPoolOptions::with_num_threads(8),
        8,
    );

    assert_eq!(options.worker_count, 2);
    assert_eq!(
        options.thread_budget_source,
        TextRasterThreadBudgetSource::TaskPoolAsyncCompute
    );
}

#[test]
fn text_raster_worker_pool_drain_accepts_atlas_independent_work_and_discards_old_faces() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![255])
        .expect("test bitmap should be valid");

    pool.publish_completion_for_test(result(1, 9, Ok(bitmap.clone())));
    pool.publish_completion_for_test(result(2, 8, Ok(bitmap.clone())));
    pool.publish_completion_for_test(result(3, 9, Ok(bitmap)));

    let drain =
        pool.drain_completed_for_face_epoch(9, TextRasterCompletionDrainBudget::new(8, usize::MAX));

    assert_eq!(drain.face_invalidated_count, 1);
    assert_eq!(drain.face_invalidated_ids, vec![TextRasterWorkId::new(2)]);
    assert_eq!(drain.accepted.len(), 2);
    assert_eq!(drain.accepted[0].id, TextRasterWorkId::new(1));
    assert_eq!(drain.accepted[1].id, TextRasterWorkId::new(3));
}

#[test]
fn text_raster_worker_pool_bounds_completion_backlog_and_releases_drain_bytes() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1)
            .with_queue_depth(3)
            .with_completion_queue_depth(3)
            .with_completion_byte_budget(2),
    );
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![64])
        .expect("test bitmap should be valid");

    assert!(pool.try_publish_completion_for_test(result(30, 1, Ok(bitmap.clone()))));
    assert!(pool.try_publish_completion_for_test(result(31, 1, Ok(bitmap.clone()))));
    assert!(
        !pool.try_publish_completion_for_test(result(32, 1, Ok(bitmap))),
        "the bounded completion queue must apply backpressure instead of growing unbounded"
    );

    let before_drain = pool.diagnostics();
    assert_eq!(before_drain.completion_backlog, 2);
    assert_eq!(before_drain.completion_backlog_bytes, 2);
    assert_eq!(before_drain.completion_backpressured, 1);

    let drain = pool.drain_completed_for_face_epoch(1, TextRasterCompletionDrainBudget::new(1, 1));
    assert_eq!(drain.accepted.len(), 1);

    let after_drain = pool.diagnostics();
    assert_eq!(after_drain.completion_backlog, 1);
    assert_eq!(after_drain.completion_backlog_bytes, 1);
}

#[test]
fn text_raster_worker_pool_normalizes_zero_completion_byte_budget() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1)
            .with_completion_queue_depth(2)
            .with_completion_byte_budget(0),
    );
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![64])
        .expect("test bitmap should be valid");

    assert_eq!(pool.options().completion_byte_budget, 1);
    assert!(pool.try_publish_completion_for_test(result(33, 1, Ok(bitmap.clone()))));
    assert!(
        !pool.try_publish_completion_for_test(result(34, 1, Ok(bitmap))),
        "the normalized byte budget must still bound the completion queue"
    );
    assert_eq!(pool.diagnostics().completion_backlog_bytes, 1);
}

#[test]
fn text_raster_worker_pool_drop_cancels_completion_backpressure() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse as face 0");
    let glyph_id = font.charmap().map('P');
    assert_ne!(glyph_id, 0, "test glyph should be present in FiraSans");

    let pool = TextRasterWorkerPool::new(
        TextRasterWorkerPoolOptions::new(1)
            .with_queue_depth(2)
            .with_completion_queue_depth(1)
            .with_completion_byte_budget(1),
    )
    .expect("one raster worker should start");
    for id in [40, 41] {
        pool.request(TextRasterWorkItem::new(
            TextRasterWorkId::new(id),
            1,
            Arc::<[u8]>::from(TEST_FONT_BYTES),
            SwashRasterRequest::alpha_outline(0, glyph_id, 18.0, true),
        ))
        .expect("raster work should enter the bounded request queue");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    while pool.diagnostics().completion_backlog == 0 {
        assert!(
            Instant::now() < deadline,
            "first raster completion should occupy the bounded completion queue"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    let drop_started = Instant::now();
    drop(pool);
    assert!(
        drop_started.elapsed() < Duration::from_secs(1),
        "drop must cancel a worker blocked by completion backpressure"
    );
}

#[test]
fn text_raster_worker_pool_frame_sampler_records_completion_deltas() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let mut sampler = TextRasterWorkerPoolFrameSampler::from_pool(&pool);
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![128])
        .expect("test bitmap should be valid");

    pool.publish_completion_for_test(result(20, 1, Ok(bitmap)));
    pool.publish_completion_for_test(result(
        21,
        1,
        Err(crate::text::raster::SwashRasterError::InvalidPxSize),
    ));

    let first_frame = sampler.sample(&pool);
    assert_eq!(first_frame.completed_delta, 2);
    assert_eq!(first_frame.failed_delta, 1);
    let second_frame = sampler.sample(&pool);
    assert_eq!(second_frame.completed_delta, 0);
    assert_eq!(second_frame.failed_delta, 0);

    let mut store = DiagnosticStore::default();
    pool.record_diagnostics(&mut store, 11);
    first_frame.record_diagnostics(&mut store, 12);
    sampler.record_diagnostics(&pool, &mut store, 13);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, TEXT_RASTER_WORKER_QUEUED_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, TEXT_RASTER_WORKER_RUNNING_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC),
        Some(2.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_history(&snapshot, TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC),
        vec![2.0, 0.0]
    );
    assert_eq!(
        diagnostic_history(&snapshot, TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC),
        vec![1.0, 0.0]
    );
}

fn result(
    id: u64,
    face_epoch: u64,
    result: Result<GlyphBitmap, crate::text::raster::SwashRasterError>,
) -> TextRasterWorkResult {
    TextRasterWorkResult {
        id: TextRasterWorkId::new(id),
        face_epoch,
        result,
    }
}

fn diagnostic_current(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

fn diagnostic_history(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> Vec<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .map(|series| series.history.iter().map(|sample| sample.value).collect())
        .unwrap_or_default()
}
