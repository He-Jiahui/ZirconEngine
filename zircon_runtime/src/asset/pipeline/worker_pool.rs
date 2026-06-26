//! Background worker pool for asset decoding.

use crossbeam_channel::{bounded, unbounded, TrySendError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::channel::{ChannelReceiver, ChannelSender};
use crate::core::runtime::tasks::{spawn_named_thread, TaskPoolOptions};
use crate::core::ZirconError;

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
    request_tx: Option<ChannelSender<AssetRequest>>,
    #[cfg(test)]
    // Keeps the request channel connected while tests exercise bounded overflow without workers.
    request_rx_guard: Option<ChannelReceiver<AssetRequest>>,
    in_flight: Arc<Mutex<HashMap<AssetRequest, usize>>>,
    diagnostics: Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    completion_tx: ChannelSender<CpuAssetPayload>,
    completion_rx: ChannelReceiver<CpuAssetPayload>,
    joins: Vec<JoinHandle<()>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetWorkerThreadBudgetSource {
    #[default]
    Explicit,
    TaskPoolIo,
}

impl AssetWorkerThreadBudgetSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::TaskPoolIo => "task_pool_io",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetWorkerPoolOptions {
    pub worker_count: usize,
    pub queue_depth: Option<usize>,
    pub thread_budget_source: AssetWorkerThreadBudgetSource,
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
    pub fn new(worker_count: usize) -> Self {
        Self {
            worker_count: worker_count.max(1),
            queue_depth: None,
            thread_budget_source: AssetWorkerThreadBudgetSource::Explicit,
        }
    }

    pub fn from_task_pool_options(
        task_pool_options: &TaskPoolOptions,
        available_parallelism: usize,
    ) -> Self {
        let thread_counts = task_pool_options.resolve_thread_counts(available_parallelism);
        Self::new(thread_counts.io_threads)
            .with_thread_budget_source(AssetWorkerThreadBudgetSource::TaskPoolIo)
    }

    pub fn with_queue_depth(mut self, queue_depth: usize) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    pub fn with_thread_budget_source(
        mut self,
        thread_budget_source: AssetWorkerThreadBudgetSource,
    ) -> Self {
        self.thread_budget_source = thread_budget_source;
        self
    }

    fn normalized(mut self) -> Self {
        self.worker_count = self.worker_count.max(1);
        self
    }
}

impl Default for AssetWorkerPoolDiagnostics {
    fn default() -> Self {
        Self {
            thread_budget_source: AssetWorkerThreadBudgetSource::Explicit,
            budgeted_threads: 0,
            in_flight: 0,
            completed: 0,
            failed: 0,
            queue_peak: 0,
        }
    }
}

impl AssetWorkerPoolDiagnostics {
    fn for_options(options: &AssetWorkerPoolOptions) -> Self {
        Self {
            thread_budget_source: options.thread_budget_source,
            budgeted_threads: options.worker_count,
            ..Self::default()
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
    pub fn new(options: AssetWorkerPoolOptions) -> Result<Self, ZirconError> {
        let options = options.normalized();
        let worker_count = options.worker_count;
        let (request_tx, request_rx) = request_channel(options.queue_depth);
        let (completion_tx, completion_rx) = unbounded();
        let in_flight = Arc::new(Mutex::new(HashMap::new()));
        let diagnostics = Arc::new(Mutex::new(AssetWorkerPoolDiagnostics::for_options(
            &options,
        )));
        let mut joins = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let request_rx = request_rx.clone();
            let completion_tx = completion_tx.clone();
            let in_flight = Arc::clone(&in_flight);
            let diagnostics = Arc::clone(&diagnostics);
            joins.push(spawn_named_thread(
                format!("zircon-asset-{worker_index}"),
                move || {
                    while let Ok(request) = request_rx.recv() {
                        let payload = process_request(request);
                        publish_completion(&completion_tx, &in_flight, &diagnostics, payload);
                    }
                },
            )?);
        }

        Ok(Self {
            options,
            request_tx: Some(request_tx),
            #[cfg(test)]
            request_rx_guard: None,
            in_flight,
            diagnostics,
            completion_tx,
            completion_rx,
            joins,
        })
    }

    #[cfg(test)]
    pub(crate) fn new_without_workers_for_test(options: AssetWorkerPoolOptions) -> Self {
        let options = options.normalized();
        let (request_tx, request_rx) = request_channel(options.queue_depth);
        let (_completion_tx, completion_rx) = unbounded();
        let diagnostics = Arc::new(Mutex::new(AssetWorkerPoolDiagnostics::for_options(
            &options,
        )));

        Self {
            options,
            request_tx: Some(request_tx),
            request_rx_guard: Some(request_rx),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            diagnostics,
            completion_tx: _completion_tx,
            completion_rx,
            joins: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn request_channel_guard_is_alive_for_test(&self) -> bool {
        self.request_rx_guard.is_some()
    }

    pub fn options(&self) -> &AssetWorkerPoolOptions {
        &self.options
    }

    pub fn request(&self, request: AssetRequest) -> Result<(), ZirconError> {
        let mut in_flight = self.lock_in_flight();
        if let Some(waiter_count) = in_flight.get_mut(&request) {
            *waiter_count += 1;
            self.record_in_flight_locked(&in_flight);
            return Ok(());
        }

        // Register the in-flight key before publishing to the worker channel so
        // a very fast worker cannot complete and remove the key before it exists.
        let queued_request = request.clone();
        in_flight.insert(request.clone(), 1);
        if let Err(error) = self
            .request_tx
            .as_ref()
            .expect("asset worker request sender alive")
            .try_send(queued_request)
        {
            in_flight.remove(&request);
            self.record_in_flight_locked(&in_flight);
            return Err(match error {
                TrySendError::Full(request) => {
                    ZirconError::ChannelSend(format!("asset request queue full: {request:?}"))
                }
                TrySendError::Disconnected(request) => {
                    ZirconError::ChannelSend(format!("asset request dropped: {request:?}"))
                }
            });
        }
        self.record_in_flight_locked(&in_flight);
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

    #[cfg(test)]
    pub(crate) fn publish_completion_for_test(&self, payload: CpuAssetPayload) {
        publish_completion(
            &self.completion_tx,
            &self.in_flight,
            &self.diagnostics,
            payload,
        );
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
}

impl Drop for AssetWorkerPool {
    fn drop(&mut self) {
        self.request_tx.take();

        for join in self.joins.drain(..) {
            let _ = join.join();
        }
    }
}

fn request_channel(
    queue_depth: Option<usize>,
) -> (ChannelSender<AssetRequest>, ChannelReceiver<AssetRequest>) {
    match queue_depth {
        Some(queue_depth) => bounded(queue_depth),
        None => unbounded(),
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
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::asset::types::TextureSource;

    use super::*;

    #[test]
    fn asset_worker_pool_accessors_recover_poisoned_locks() {
        let pool = AssetWorkerPool::new_without_workers_for_test(
            AssetWorkerPoolOptions::new(1).with_queue_depth(1),
        );
        let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.in_flight.lock().unwrap();
            panic!("poison asset worker in-flight lock");
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.diagnostics.lock().unwrap();
            panic!("poison asset worker diagnostics lock");
        }));

        pool.request(request.clone())
            .expect("request should recover poisoned locks");
        assert_eq!(pool.diagnostics().in_flight, 1);

        pool.publish_completion_for_test(CpuAssetPayload::Failure {
            request,
            message: "decode failed".to_string(),
        });
        let diagnostics = pool.diagnostics();
        assert_eq!(diagnostics.in_flight, 0);
        assert_eq!(diagnostics.completed, 1);
        assert_eq!(diagnostics.failed, 1);
    }
}
