use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuTexturePayload, TextureSource,
};
use crate::asset::pipeline::worker_pool::{
    AssetWorkerPool, AssetWorkerPoolOptions, AssetWorkerThreadBudgetSource,
    ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC, ASSET_WORKER_COMPLETED_DIAGNOSTIC,
    ASSET_WORKER_FAILED_DIAGNOSTIC, ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
};
use crate::core::diagnostics::DiagnosticStore;
use crate::core::runtime::tasks::TaskPoolOptions;

#[test]
fn worker_pool_completes_builtin_texture_requests() {
    let pool = AssetWorkerPool::new(AssetWorkerPoolOptions::new(1)).unwrap();
    let completions = pool.completion_receiver();

    pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    let payload = completions.recv().unwrap();
    match payload {
        CpuAssetPayload::Texture(texture) => {
            assert_eq!(texture.source, TextureSource::BuiltinChecker);
            assert_eq!(
                texture.rgba.len(),
                texture.width as usize * texture.height as usize * 4
            );
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[test]
fn worker_pool_unbounded_mode_is_explicit_opt_in() {
    let options = AssetWorkerPoolOptions::new(1);
    let pool = AssetWorkerPool::new(options.clone()).unwrap();

    assert_eq!(pool.options(), &options);
    assert_eq!(pool.options().queue_depth, None);
    assert_eq!(
        pool.options().thread_budget_source,
        AssetWorkerThreadBudgetSource::Explicit
    );
}

#[test]
fn worker_pool_options_can_derive_threads_from_runtime_io_budget() {
    let options =
        AssetWorkerPoolOptions::from_task_pool_options(&TaskPoolOptions::with_num_threads(8), 8);

    assert_eq!(options.worker_count, 2);
    assert_eq!(
        options.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
}

#[test]
fn project_asset_manager_default_workers_use_runtime_io_budget_source() {
    let manager = ProjectAssetManager::default();
    let available_parallelism = std::thread::available_parallelism().map_or(1, |value| value.get());
    let expected_options = AssetWorkerPoolOptions::from_task_pool_options(
        &TaskPoolOptions::default(),
        available_parallelism,
    );

    assert_eq!(
        manager.default_worker_count(),
        expected_options.worker_count
    );
    assert_eq!(
        manager.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );

    let explicit_manager = ProjectAssetManager::new(3);
    assert_eq!(explicit_manager.default_worker_count(), 3);
    assert_eq!(
        explicit_manager.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::Explicit
    );
}

#[test]
fn worker_pool_bounded_queue_rejects_overflow_with_explicit_error() {
    let pool = AssetWorkerPool::new_without_workers_for_test(
        AssetWorkerPoolOptions::new(1).with_queue_depth(0),
    );

    let error = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .expect_err("zero-depth queue without a waiting worker must reject");

    assert!(
        error.to_string().contains("asset request queue full"),
        "unexpected error: {error}"
    );
    assert_eq!(pool.diagnostics().in_flight, 0);
    assert_eq!(pool.diagnostics().queue_peak, 0);
}

#[test]
fn concurrent_requests_for_same_asset_decode_once_and_notify_all() {
    let pool = AssetWorkerPool::new_without_workers_for_test(
        AssetWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let completions = pool.completion_receiver();
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    pool.request(request.clone()).unwrap();
    pool.request(request).unwrap();
    let overflow = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinGrid))
        .expect_err("different request must still see the full bounded queue");
    assert!(
        overflow.to_string().contains("asset request queue full"),
        "unexpected error: {overflow}"
    );

    pool.publish_completion_for_test(CpuAssetPayload::Texture(CpuTexturePayload {
        source: TextureSource::BuiltinChecker,
        width: 1,
        height: 1,
        rgba: vec![255, 0, 0, 255],
    }));

    for _ in 0..2 {
        match completions.recv().unwrap() {
            CpuAssetPayload::Texture(texture) => {
                assert_eq!(texture.source, TextureSource::BuiltinChecker);
            }
            other => panic!("unexpected payload: {other:?}"),
        }
    }
    assert!(completions.try_recv().is_err());
}

#[test]
fn worker_pool_diagnostics_track_in_flight_and_failure_counts() {
    let pool = AssetWorkerPool::new_without_workers_for_test(
        AssetWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    pool.request(request.clone()).unwrap();
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().queue_peak, 1);

    pool.publish_completion_for_test(CpuAssetPayload::Failure {
        request,
        message: "decode failed".to_string(),
    });
    let diagnostics = pool.diagnostics();
    assert_eq!(
        diagnostics.thread_budget_source,
        AssetWorkerThreadBudgetSource::Explicit
    );
    assert_eq!(diagnostics.budgeted_threads, 1);
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.completed, 1);
    assert_eq!(diagnostics.failed, 1);
    assert_eq!(diagnostics.queue_peak, 1);

    let mut store = DiagnosticStore::default();
    pool.record_diagnostics(&mut store, 7);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_COMPLETED_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_FAILED_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC),
        Some(1.0)
    );
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
