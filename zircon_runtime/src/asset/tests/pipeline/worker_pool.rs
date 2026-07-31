use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Sender};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::pipeline::types::{AssetRequest, CpuAssetPayload, TextureSource};
use crate::asset::pipeline::worker_pool::{
    AssetWorkerCompletionError, AssetWorkerCompletionTicket, AssetWorkerPool,
    AssetWorkerPoolFrameSampler, AssetWorkerPoolOptions, AssetWorkerThreadBudgetSource,
    ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC, ASSET_WORKER_CANCEL_WALL_SAMPLES_DIAGNOSTIC,
    ASSET_WORKER_CANCEL_WALL_TOTAL_MS_DIAGNOSTIC, ASSET_WORKER_COMPLETED_DIAGNOSTIC,
    ASSET_WORKER_COMPLETION_BYTES_DIAGNOSTIC, ASSET_WORKER_DROP_WALL_SAMPLES_DIAGNOSTIC,
    ASSET_WORKER_DROP_WALL_TOTAL_MS_DIAGNOSTIC, ASSET_WORKER_FAILED_DIAGNOSTIC,
    ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC, ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC,
    ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC, ASSET_WORKER_PAYLOAD_CLONE_BYTES_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_AGE_SAMPLES_DIAGNOSTIC, ASSET_WORKER_QUEUE_AGE_TOTAL_MS_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
};
use crate::core::diagnostics::DiagnosticStore;
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskTimer};
use crate::core::CoreError;

mod diagnostics;
mod lifecycle;
mod single_flight;
mod task_pool;

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

fn receive_completion(ticket: &AssetWorkerCompletionTicket) -> Arc<CpuAssetPayload> {
    ticket
        .wait_timeout(Duration::from_secs(2))
        .expect("asset request should complete on the runtime IO pool")
}

fn wait_for_admission(
    pool: &AssetWorkerPool,
    request: AssetRequest,
) -> AssetWorkerCompletionTicket {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match pool.request(request.clone()) {
            Ok(ticket) => return ticket,
            Err(error) if error.to_string().contains("asset request queue full") => {
                assert!(
                    Instant::now() < deadline,
                    "cancelled task closure should eventually release its admission charge"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("unexpected asset worker admission error: {error}"),
        }
    }
}

fn wait_for_completed(pool: &AssetWorkerPool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while pool.diagnostics().completed == 0 {
        assert!(
            Instant::now() < deadline,
            "asset worker should publish its completion before the test deadline"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn wait_for_expiry(pool: &AssetWorkerPool, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while pool.diagnostics().expired == 0 {
        assert!(
            Instant::now() < deadline,
            "the shared runtime timer should enforce request and completion age budgets"
        );
        std::thread::sleep(Duration::from_millis(1));
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
