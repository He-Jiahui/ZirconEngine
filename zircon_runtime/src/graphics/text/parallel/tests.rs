use std::sync::Arc;
use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::channel::wait_for;
use crate::core::math::{UVec2, Vec2};
use crate::core::runtime::tasks::TaskPoolOptions;
use crate::graphics::text::raster::{GlyphBitmap, SwashRasterRequest};
use ::swash::FontRef;

use super::raster_pool::{
    TextRasterThreadBudgetSource, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkTarget, TextRasterWorkerPool, TextRasterWorkerPoolFrameSampler,
    TextRasterWorkerPoolOptions, TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
    TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC, TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
    TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC, TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC,
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
    let target = TextRasterWorkTarget::new(7, 3);
    pool.request(TextRasterWorkItem::new(
        TextRasterWorkId::new(1),
        target,
        Arc::<[u8]>::from(TEST_FONT_BYTES),
        SwashRasterRequest::alpha_outline(0, glyph_id, 18.0, true),
    ))
    .unwrap();

    let result = wait_for(&pool.completion_receiver(), Duration::from_secs(5))
        .expect("text raster worker should publish a completion");
    let bitmap = result.result.expect("worker should rasterize alpha glyph");

    assert_eq!(result.id, TextRasterWorkId::new(1));
    assert_eq!(result.target, target);
    assert!(bitmap.size.x > 0);
    assert!(bitmap.size.y > 0);
    assert!(bitmap.has_expected_data_len());
    assert!(
        bitmap.data.iter().any(|coverage| *coverage > 0),
        "worker result should contain glyph coverage"
    );

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completed, 1);
    assert_eq!(diagnostics.failed, 0);
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.queue_peak, 1);
}

#[test]
fn text_raster_worker_pool_bounded_queue_rejects_overflow_without_workers() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(0),
    );
    assert!(pool.request_channel_guard_is_alive_for_test());

    let error = pool
        .request(TextRasterWorkItem::new(
            TextRasterWorkId::new(10),
            TextRasterWorkTarget::new(1, 1),
            Arc::<[u8]>::from(TEST_FONT_BYTES),
            SwashRasterRequest::alpha_outline(0, 1, 16.0, true),
        ))
        .expect_err("zero-depth queue without a waiting worker must reject");

    assert!(
        error.to_string().contains("text raster work queue full"),
        "unexpected error: {error}"
    );
    assert_eq!(pool.diagnostics().in_flight, 0);
    assert_eq!(pool.diagnostics().queue_peak, 0);
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
fn text_raster_worker_pool_drain_discards_stale_page_generation_and_face_epoch() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![255])
        .expect("test bitmap should be valid");

    pool.publish_completion_for_test(result(
        1,
        TextRasterWorkTarget::new(1, 9),
        Ok(bitmap.clone()),
    ));
    pool.publish_completion_for_test(result(
        2,
        TextRasterWorkTarget::new(2, 8),
        Ok(bitmap.clone()),
    ));
    pool.publish_completion_for_test(result(3, TextRasterWorkTarget::new(2, 9), Ok(bitmap)));

    let drain = pool.drain_completed_for_target(TextRasterWorkTarget::new(2, 9));

    assert_eq!(drain.stale_page_generation_count, 1);
    assert_eq!(drain.face_invalidated_count, 1);
    assert_eq!(
        drain.stale_page_generation_ids,
        vec![TextRasterWorkId::new(1)]
    );
    assert_eq!(drain.face_invalidated_ids, vec![TextRasterWorkId::new(2)]);
    assert_eq!(drain.accepted.len(), 1);
    assert_eq!(drain.accepted[0].id, TextRasterWorkId::new(3));
}

#[test]
fn text_raster_worker_pool_frame_sampler_records_completion_deltas() {
    let pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let mut sampler = TextRasterWorkerPoolFrameSampler::from_pool(&pool);
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![128])
        .expect("test bitmap should be valid");

    pool.publish_completion_for_test(result(20, TextRasterWorkTarget::new(1, 1), Ok(bitmap)));
    pool.publish_completion_for_test(result(
        21,
        TextRasterWorkTarget::new(1, 1),
        Err(crate::graphics::text::raster::SwashRasterError::InvalidPxSize),
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
    target: TextRasterWorkTarget,
    result: Result<GlyphBitmap, crate::graphics::text::raster::SwashRasterError>,
) -> TextRasterWorkResult {
    TextRasterWorkResult {
        id: TextRasterWorkId::new(id),
        target,
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
