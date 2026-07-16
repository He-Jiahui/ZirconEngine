use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::pipeline::types::{AssetRequest, CpuAssetPayload, TextureSource};
use crate::asset::pipeline::worker_pool::{
    AssetWorkerPool, AssetWorkerPoolFrameSampler, AssetWorkerPoolOptions,
    AssetWorkerThreadBudgetSource, ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
    ASSET_WORKER_COMPLETED_DIAGNOSTIC, ASSET_WORKER_FAILED_DIAGNOSTIC,
    ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC, ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC,
    ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC, ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
};
use crate::core::diagnostics::DiagnosticStore;
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor, TaskPoolKind};

#[test]
fn worker_pool_completes_builtin_texture_requests_on_the_runtime_io_pool() {
    let io_pool = single_worker_io_pool();
    let pool = AssetWorkerPool::new(io_pool.clone(), AssetWorkerPoolOptions::new());
    let completions = pool.completion_receiver();

    assert_eq!(pool.task_pool().kind(), TaskPoolKind::Io);
    assert!(pool.task_pool().shares_execution_owner_with(&io_pool));

    pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    match receive_completion(&completions) {
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
    let options = AssetWorkerPoolOptions::new();
    let pool = AssetWorkerPool::new(single_worker_io_pool(), options.clone());

    assert_eq!(pool.options(), &options);
    assert_eq!(pool.options().queue_depth, None);
    assert_eq!(
        pool.diagnostics().thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
}

#[test]
fn project_asset_manager_uses_the_injected_runtime_io_pool() {
    let io_pool = single_worker_io_pool();
    let manager = ProjectAssetManager::new(io_pool.clone());
    let (pool, mut sampler) = manager.spawn_worker_pool_with_frame_sampler();

    assert!(manager
        .worker_task_pool()
        .shares_execution_owner_with(&io_pool));
    assert!(pool.task_pool().shares_execution_owner_with(&io_pool));
    assert_eq!(manager.default_worker_count(), io_pool.parallelism());
    assert_eq!(
        manager.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );

    let frame = sampler.sample(&pool);
    assert_eq!(
        frame.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
    assert_eq!(frame.budgeted_threads, 1);
    assert_eq!(frame.in_flight, 0);
    assert_eq!(frame.completed_delta, 0);
    assert_eq!(frame.failed_delta, 0);
}

#[test]
fn project_asset_manager_defaults_share_the_process_io_pool() {
    let first = ProjectAssetManager::default();
    let second = ProjectAssetManager::default();

    assert!(first
        .worker_task_pool()
        .shares_execution_owner_with(second.worker_task_pool()));
    assert_eq!(
        first.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
}

#[test]
fn worker_pool_bounded_queue_rejects_overflow_with_explicit_error() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(0));
    let completions = pool.completion_receiver();

    pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();
    let error = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinGrid))
        .expect_err("a second unique request must exceed one IO worker with zero queue depth");

    assert!(
        error.to_string().contains("asset request queue full"),
        "unexpected error: {error}"
    );
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().queue_peak, 1);

    release.send(()).unwrap();
    receive_completion(&completions);
}

#[test]
fn concurrent_requests_for_same_asset_decode_once_and_notify_all() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(0));
    let completions = pool.completion_receiver();
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    pool.request(request.clone()).unwrap();
    pool.request(request).unwrap();
    let overflow = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinGrid))
        .expect_err("a different request must still see the full bounded queue");
    assert!(
        overflow.to_string().contains("asset request queue full"),
        "unexpected error: {overflow}"
    );

    release.send(()).unwrap();
    for _ in 0..2 {
        match receive_completion(&completions) {
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
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let completions = pool.completion_receiver();
    let request = missing_texture_request("diagnostic");

    pool.request(request).unwrap();
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().queue_peak, 1);
    release.send(()).unwrap();
    assert!(matches!(
        receive_completion(&completions),
        CpuAssetPayload::Failure { .. }
    ));

    let diagnostics = pool.diagnostics();
    assert_eq!(
        diagnostics.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
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

#[test]
fn worker_pool_frame_sampler_records_per_frame_completion_deltas() {
    let pool = AssetWorkerPool::new(single_worker_io_pool(), AssetWorkerPoolOptions::new());
    let completions = pool.completion_receiver();
    let mut sampler = AssetWorkerPoolFrameSampler::from_pool(&pool);
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    pool.request(request.clone()).unwrap();
    pool.request(request).unwrap();
    pool.request(missing_texture_request("frame-sampler"))
        .unwrap();
    for _ in 0..3 {
        receive_completion(&completions);
    }

    let first_frame = sampler.sample(&pool);
    assert_eq!(
        first_frame.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
    assert_eq!(first_frame.budgeted_threads, 1);
    assert_eq!(first_frame.in_flight, 0);
    assert_eq!(first_frame.completed_delta, 3);
    assert_eq!(first_frame.failed_delta, 1);

    let second_frame = sampler.sample(&pool);
    assert_eq!(second_frame.completed_delta, 0);
    assert_eq!(second_frame.failed_delta, 0);

    let mut store = DiagnosticStore::default();
    first_frame.record_diagnostics(&mut store, 11);
    sampler.record_diagnostics(&pool, &mut store, 12);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_history(&snapshot, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC),
        vec![3.0, 0.0]
    );
    assert_eq!(
        diagnostic_history(&snapshot, ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC),
        vec![1.0, 0.0]
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC),
        Some(1.0)
    );
}

#[test]
fn dropping_worker_pool_waits_for_its_runtime_io_jobs() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();
    let (drop_started_tx, drop_started_rx) = bounded::<()>(1);
    let (dropped_tx, dropped_rx) = bounded::<()>(1);

    let drop_thread = std::thread::spawn(move || {
        drop_started_tx.send(()).unwrap();
        drop(pool);
        dropped_tx.send(()).unwrap();
    });

    drop_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("drop thread should start before checking the pending-job wait");
    assert!(dropped_rx.recv_timeout(Duration::from_millis(25)).is_err());
    release.send(()).unwrap();
    dropped_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("worker pool should finish after the runtime IO task completes");
    drop_thread.join().unwrap();
}

#[test]
fn dropping_worker_pool_on_its_io_worker_does_not_deadlock_pending_jobs() {
    let io_pool = single_worker_io_pool();
    let pool = AssetWorkerPool::new(io_pool.clone(), AssetWorkerPoolOptions::new());
    let completions = pool.completion_receiver();
    let (dropped_tx, dropped_rx) = bounded::<()>(1);

    io_pool.spawn(move || {
        pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
            .unwrap();
        drop(pool);
        dropped_tx.send(()).unwrap();
    });

    dropped_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("dropping on the only IO worker must return before the queued request runs");
    assert!(matches!(
        receive_completion(&completions),
        CpuAssetPayload::Texture(_)
    ));
}

fn single_worker_io_pool() -> TaskPool {
    TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1))
}

fn occupy_io_pool(pool: &TaskPool) -> Sender<()> {
    let (started_tx, started_rx) = bounded::<()>(0);
    let (release_tx, release_rx) = bounded::<()>(0);
    pool.spawn(move || {
        started_tx.send(()).unwrap();
        release_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("test should release the occupied IO worker");
    });
    started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("IO worker should start the blocker");
    release_tx
}

fn missing_texture_request(label: &str) -> AssetRequest {
    AssetRequest::Texture(TextureSource::Path(format!(
        "missing-runtime11-worker-{label}.png"
    )))
}

fn receive_completion(receiver: &Receiver<CpuAssetPayload>) -> CpuAssetPayload {
    receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("asset request should complete on the runtime IO pool")
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
