//! Runtime IO-pool orchestration for CPU-side asset decoding.

use crossbeam_channel::unbounded;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Condvar;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::channel::{ChannelReceiver, ChannelSender};
use crate::core::runtime::tasks::{TaskPool, TaskPoolKind};
use crate::core::{CoreError, CoreResult};

use crate::asset::load::{mesh, texture};
use crate::asset::types::{AssetRequest, CpuAssetPayload};

pub const ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "asset.worker.in_flight";
pub const ASSET_WORKER_COMPLETED_DIAGNOSTIC: &str = "asset.worker.completed";
pub const ASSET_WORKER_FAILED_DIAGNOSTIC: &str = "asset.worker.failed";
pub const ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "asset.worker.queue_peak";
pub const ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC: &str = "asset.worker.budgeted_threads";
pub const ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC: &str = "asset.worker.frame_completed";
pub const ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC: &str = "asset.worker.frame_failed";

pub struct AssetWorkerPool {
    options: AssetWorkerPoolOptions,
    task_pool: TaskPool,
    in_flight: Arc<Mutex<HashMap<AssetRequest, usize>>>,
    diagnostics: Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    completion_tx: ChannelSender<CpuAssetPayload>,
    completion_rx: ChannelReceiver<CpuAssetPayload>,
    lifecycle: Arc<AssetWorkerLifecycle>,
}

struct AssetWorkerLifecycle {
    pending_jobs: Mutex<usize>,
    pending_jobs_changed: Condvar,
}

struct PendingJobGuard {
    lifecycle: Arc<AssetWorkerLifecycle>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetWorkerThreadBudgetSource {
    #[default]
    TaskPoolIo,
}

impl AssetWorkerThreadBudgetSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TaskPoolIo => "task_pool_io",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetWorkerPoolOptions {
    pub queue_depth: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetWorkerPoolDiagnostics {
    pub thread_budget_source: AssetWorkerThreadBudgetSource,
    pub budgeted_threads: usize,
    pub in_flight: usize,
    pub completed: u64,
    pub failed: u64,
    pub queue_peak: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetWorkerPoolFrameDiagnostics {
    pub thread_budget_source: AssetWorkerThreadBudgetSource,
    pub budgeted_threads: usize,
    pub in_flight: usize,
    pub completed_delta: u64,
    pub failed_delta: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetWorkerPoolFrameSampler {
    last_completed: u64,
    last_failed: u64,
}

impl AssetWorkerPoolOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_queue_depth(mut self, queue_depth: usize) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }
}

impl AssetWorkerPoolDiagnostics {
    fn for_task_pool(task_pool: &TaskPool) -> Self {
        Self {
            thread_budget_source: AssetWorkerThreadBudgetSource::TaskPoolIo,
            budgeted_threads: task_pool.parallelism(),
            in_flight: 0,
            completed: 0,
            failed: 0,
            queue_peak: 0,
        }
    }
}

impl AssetWorkerPoolFrameDiagnostics {
    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        store.record(
            ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
            frame_index,
            self.in_flight as f64,
            Some("request"),
            ["asset", "worker"],
        );
        store.record(
            ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
            frame_index,
            self.budgeted_threads as f64,
            Some("thread"),
            [
                "asset",
                "worker",
                "budget",
                self.thread_budget_source.as_str(),
            ],
        );
        store.record(
            ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
            frame_index,
            self.completed_delta as f64,
            Some("request"),
            ["asset", "worker", "frame"],
        );
        store.record(
            ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC,
            frame_index,
            self.failed_delta as f64,
            Some("request"),
            ["asset", "worker", "frame"],
        );
    }
}

impl AssetWorkerPoolFrameSampler {
    pub fn from_pool(pool: &AssetWorkerPool) -> Self {
        let diagnostics = pool.diagnostics();
        Self {
            last_completed: diagnostics.completed,
            last_failed: diagnostics.failed,
        }
    }

    pub fn sample(&mut self, pool: &AssetWorkerPool) -> AssetWorkerPoolFrameDiagnostics {
        let diagnostics = pool.diagnostics();
        let frame = AssetWorkerPoolFrameDiagnostics {
            thread_budget_source: diagnostics.thread_budget_source,
            budgeted_threads: diagnostics.budgeted_threads,
            in_flight: diagnostics.in_flight,
            completed_delta: diagnostics.completed.saturating_sub(self.last_completed),
            failed_delta: diagnostics.failed.saturating_sub(self.last_failed),
        };
        self.last_completed = diagnostics.completed;
        self.last_failed = diagnostics.failed;
        frame
    }

    pub fn record_diagnostics(
        &mut self,
        pool: &AssetWorkerPool,
        store: &mut DiagnosticStore,
        frame_index: u64,
    ) {
        self.sample(pool).record_diagnostics(store, frame_index);
    }
}

impl AssetWorkerPool {
    pub fn new(task_pool: TaskPool, options: AssetWorkerPoolOptions) -> Self {
        assert_eq!(
            task_pool.kind(),
            TaskPoolKind::Io,
            "AssetWorkerPool requires the runtime IO task pool"
        );
        let (completion_tx, completion_rx) = unbounded();
        let diagnostics = Arc::new(Mutex::new(AssetWorkerPoolDiagnostics::for_task_pool(
            &task_pool,
        )));

        Self {
            options,
            task_pool,
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            diagnostics,
            completion_tx,
            completion_rx,
            lifecycle: Arc::new(AssetWorkerLifecycle {
                pending_jobs: Mutex::new(0),
                pending_jobs_changed: Condvar::new(),
            }),
        }
    }

    pub fn options(&self) -> &AssetWorkerPoolOptions {
        &self.options
    }

    pub fn task_pool(&self) -> &TaskPool {
        &self.task_pool
    }

    pub fn request(&self, request: AssetRequest) -> CoreResult<()> {
        let mut in_flight = self.lock_in_flight();
        if let Some(waiter_count) = in_flight.get_mut(&request) {
            *waiter_count += 1;
            self.record_in_flight_locked(&in_flight);
            return Ok(());
        }

        if self.unique_request_capacity_reached(in_flight.len()) {
            return Err(CoreError::ChannelSend(format!(
                "asset request queue full: {request:?}"
            )));
        }

        in_flight.insert(request.clone(), 1);
        self.record_in_flight_locked(&in_flight);
        drop(in_flight);

        self.begin_pending_job();
        let task_pool = self.task_pool.clone();
        let completion_tx = self.completion_tx.clone();
        let in_flight = Arc::clone(&self.in_flight);
        let diagnostics = Arc::clone(&self.diagnostics);
        let lifecycle = Arc::clone(&self.lifecycle);
        task_pool.spawn(move || {
            let _pending_job = PendingJobGuard { lifecycle };
            let panic_request = request.clone();
            let payload = catch_unwind(AssertUnwindSafe(|| process_request(request))).unwrap_or(
                CpuAssetPayload::Failure {
                    request: panic_request,
                    message: "asset worker task panicked".to_string(),
                },
            );
            publish_completion(&completion_tx, &in_flight, &diagnostics, payload);
        });
        Ok(())
    }

    pub fn completion_receiver(&self) -> ChannelReceiver<CpuAssetPayload> {
        self.completion_rx.clone()
    }

    pub fn diagnostics(&self) -> AssetWorkerPoolDiagnostics {
        *self.lock_diagnostics()
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        let diagnostics = self.diagnostics();
        for (path, value) in [
            (
                ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
                diagnostics.in_flight as f64,
            ),
            (
                ASSET_WORKER_COMPLETED_DIAGNOSTIC,
                diagnostics.completed as f64,
            ),
            (ASSET_WORKER_FAILED_DIAGNOSTIC, diagnostics.failed as f64),
            (
                ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
                diagnostics.queue_peak as f64,
            ),
        ] {
            store.record(
                path,
                frame_index,
                value,
                Some("request"),
                ["asset", "worker"],
            );
        }
        store.record(
            ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
            frame_index,
            diagnostics.budgeted_threads as f64,
            Some("thread"),
            [
                "asset",
                "worker",
                "budget",
                diagnostics.thread_budget_source.as_str(),
            ],
        );
    }

    fn unique_request_capacity_reached(&self, unique_in_flight: usize) -> bool {
        self.options.queue_depth.is_some_and(|queue_depth| {
            let capacity = self.task_pool.parallelism().saturating_add(queue_depth);
            unique_in_flight >= capacity
        })
    }

    fn begin_pending_job(&self) {
        *lock_pending_jobs(&self.lifecycle) += 1;
    }

    fn record_in_flight_locked(&self, in_flight: &HashMap<AssetRequest, usize>) {
        let in_flight_count = total_waiter_count(in_flight);
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = in_flight_count;
        diagnostics.queue_peak = diagnostics.queue_peak.max(in_flight_count);
    }

    fn lock_in_flight(&self) -> MutexGuard<'_, HashMap<AssetRequest, usize>> {
        lock_in_flight_map(&self.in_flight)
    }

    fn lock_diagnostics(&self) -> MutexGuard<'_, AssetWorkerPoolDiagnostics> {
        lock_worker_diagnostics(&self.diagnostics)
    }

    fn wait_for_pending_jobs(&self) {
        let mut pending_jobs = lock_pending_jobs(&self.lifecycle);
        while *pending_jobs > 0 {
            pending_jobs = self
                .lifecycle
                .pending_jobs_changed
                .wait(pending_jobs)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

impl Drop for AssetWorkerPool {
    fn drop(&mut self) {
        if !self.task_pool.is_current_worker() {
            self.wait_for_pending_jobs();
        }
    }
}

impl Drop for PendingJobGuard {
    fn drop(&mut self) {
        let mut pending_jobs = lock_pending_jobs(&self.lifecycle);
        *pending_jobs = pending_jobs.saturating_sub(1);
        if *pending_jobs == 0 {
            self.lifecycle.pending_jobs_changed.notify_all();
        }
    }
}

fn publish_completion(
    completion_tx: &ChannelSender<CpuAssetPayload>,
    in_flight: &Mutex<HashMap<AssetRequest, usize>>,
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
    payload: CpuAssetPayload,
) {
    let request = request_for_payload(&payload);
    let (waiter_count, remaining_waiters) = {
        let mut in_flight = lock_in_flight_map(in_flight);
        let waiter_count = in_flight.remove(&request).unwrap_or(1);
        (waiter_count, total_waiter_count(&in_flight))
    };
    {
        let mut diagnostics = lock_worker_diagnostics(diagnostics);
        diagnostics.in_flight = remaining_waiters;
        diagnostics.completed += waiter_count as u64;
        if matches!(payload, CpuAssetPayload::Failure { .. }) {
            diagnostics.failed += waiter_count as u64;
        }
    }
    for _ in 0..waiter_count {
        let _ = completion_tx.send(payload.clone());
    }
}

fn lock_in_flight_map(
    in_flight: &Mutex<HashMap<AssetRequest, usize>>,
) -> MutexGuard<'_, HashMap<AssetRequest, usize>> {
    in_flight
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_worker_diagnostics(
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
) -> MutexGuard<'_, AssetWorkerPoolDiagnostics> {
    diagnostics
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_pending_jobs(lifecycle: &AssetWorkerLifecycle) -> MutexGuard<'_, usize> {
    lifecycle
        .pending_jobs
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn total_waiter_count(in_flight: &HashMap<AssetRequest, usize>) -> usize {
    in_flight.values().sum()
}

fn request_for_payload(payload: &CpuAssetPayload) -> AssetRequest {
    match payload {
        CpuAssetPayload::Texture(texture) => AssetRequest::Texture(texture.source.clone()),
        CpuAssetPayload::Mesh(mesh) => AssetRequest::Mesh(mesh.source.clone()),
        CpuAssetPayload::Failure { request, .. } => request.clone(),
    }
}

fn process_request(request: AssetRequest) -> CpuAssetPayload {
    match request {
        AssetRequest::Texture(source) => match texture::load_texture(&source) {
            Ok(texture) => CpuAssetPayload::Texture(texture),
            Err(error) => CpuAssetPayload::Failure {
                request: AssetRequest::Texture(source),
                message: error.to_string(),
            },
        },
        AssetRequest::Mesh(source) => match mesh::load_mesh(&source) {
            Ok(mesh) => CpuAssetPayload::Mesh(mesh),
            Err(error) => CpuAssetPayload::Failure {
                request: AssetRequest::Mesh(source),
                message: error.to_string(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use crossbeam_channel::TryRecvError;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::time::Duration;

    use crate::asset::types::TextureSource;
    use crate::core::runtime::tasks::TaskPoolDescriptor;

    use super::*;

    #[test]
    fn asset_worker_pool_accessors_recover_poisoned_locks() {
        let pool = AssetWorkerPool::new(
            TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1)),
            AssetWorkerPoolOptions::new(),
        );
        let completions = pool.completion_receiver();
        let request = AssetRequest::Texture(TextureSource::Path(
            "missing-poison-recovery-texture.png".to_string(),
        ));

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.in_flight.lock().unwrap();
            panic!("poison asset worker in-flight lock");
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.diagnostics.lock().unwrap();
            panic!("poison asset worker diagnostics lock");
        }));

        pool.request(request)
            .expect("request should recover poisoned locks");

        assert!(matches!(
            completions.recv_timeout(Duration::from_secs(2)),
            Ok(CpuAssetPayload::Failure { .. })
        ));
        let diagnostics = pool.diagnostics();
        assert_eq!(diagnostics.in_flight, 0);
        assert_eq!(diagnostics.completed, 1);
        assert_eq!(diagnostics.failed, 1);
        assert!(matches!(completions.try_recv(), Err(TryRecvError::Empty)));
    }
}
