//! Worker queue for CPU glyph rasterization.

use crossbeam_channel::{bounded, unbounded, TrySendError};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::channel::{ChannelReceiver, ChannelSender};
use crate::core::runtime::tasks::{spawn_named_thread, TaskPoolOptions};
use crate::core::{CoreError, CoreResult};
use crate::text::raster::{GlyphBitmap, SwashRasterError, SwashRasterRequest, SwashRasterizer};

pub(crate) const TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "text.raster.worker.in_flight";
pub(crate) const TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC: &str = "text.raster.worker.completed";
pub(crate) const TEXT_RASTER_WORKER_FAILED_DIAGNOSTIC: &str = "text.raster.worker.failed";
pub(crate) const TEXT_RASTER_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "text.raster.worker.queue_peak";
pub(crate) const TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC: &str =
    "text.raster.worker.budgeted_threads";
pub(crate) const TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC: &str =
    "text.raster.worker.frame_completed";
pub(crate) const TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC: &str =
    "text.raster.worker.frame_failed";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextRasterWorkId(u64);

#[derive(Clone, Debug)]
pub(crate) struct TextRasterWorkItem {
    pub(crate) id: TextRasterWorkId,
    // Raster output enters the face-owned source cache before atlas allocation. Page generations
    // are therefore validated later by atlas staging/upload, not by this worker boundary.
    pub(crate) face_epoch: u64,
    pub(crate) font_data: Arc<[u8]>,
    pub(crate) request: SwashRasterRequest,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextRasterWorkResult {
    pub(crate) id: TextRasterWorkId,
    // A font-face change invalidates raster bytes; atlas page churn does not.
    pub(crate) face_epoch: u64,
    pub(crate) result: Result<GlyphBitmap, SwashRasterError>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextRasterThreadBudgetSource {
    #[default]
    Explicit,
    TaskPoolAsyncCompute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolOptions {
    pub(crate) worker_count: usize,
    pub(crate) queue_depth: Option<usize>,
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolDiagnostics {
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
    pub(crate) budgeted_threads: usize,
    pub(crate) in_flight: usize,
    pub(crate) completed: u64,
    pub(crate) failed: u64,
    pub(crate) queue_peak: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolFrameDiagnostics {
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
    pub(crate) budgeted_threads: usize,
    pub(crate) in_flight: usize,
    pub(crate) completed_delta: u64,
    pub(crate) failed_delta: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolFrameSampler {
    last_completed: u64,
    last_failed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextRasterWorkDisposition {
    Accepted,
    InvalidatedFace,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextRasterCompletionDrain {
    pub(crate) accepted: Vec<TextRasterWorkResult>,
    pub(crate) face_invalidated_ids: Vec<TextRasterWorkId>,
    pub(crate) face_invalidated_count: usize,
}

pub(crate) struct TextRasterWorkerPool {
    options: TextRasterWorkerPoolOptions,
    request_tx: Option<ChannelSender<TextRasterWorkItem>>,
    #[cfg(test)]
    request_rx_guard: Option<ChannelReceiver<TextRasterWorkItem>>,
    in_flight: Arc<Mutex<HashSet<TextRasterWorkId>>>,
    diagnostics: Arc<Mutex<TextRasterWorkerPoolDiagnostics>>,
    completion_tx: ChannelSender<TextRasterWorkResult>,
    completion_rx: ChannelReceiver<TextRasterWorkResult>,
    joins: Vec<JoinHandle<()>>,
}

impl TextRasterWorkId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }
}

impl TextRasterWorkItem {
    pub(crate) fn new(
        id: TextRasterWorkId,
        face_epoch: u64,
        font_data: Arc<[u8]>,
        request: SwashRasterRequest,
    ) -> Self {
        Self {
            id,
            face_epoch,
            font_data,
            request,
        }
    }
}

impl TextRasterWorkResult {
    pub(crate) fn disposition_for_face_epoch(
        &self,
        live_face_epoch: u64,
    ) -> TextRasterWorkDisposition {
        if self.face_epoch != live_face_epoch {
            TextRasterWorkDisposition::InvalidatedFace
        } else {
            TextRasterWorkDisposition::Accepted
        }
    }
}

impl TextRasterThreadBudgetSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::TaskPoolAsyncCompute => "task_pool_async_compute",
        }
    }
}

impl TextRasterWorkerPoolOptions {
    pub(crate) fn new(worker_count: usize) -> Self {
        Self {
            worker_count: worker_count.max(1),
            queue_depth: None,
            thread_budget_source: TextRasterThreadBudgetSource::Explicit,
        }
    }

    pub(crate) fn from_task_pool_options(
        task_pool_options: &TaskPoolOptions,
        available_parallelism: usize,
    ) -> Self {
        let thread_counts = task_pool_options.resolve_thread_counts(available_parallelism);
        Self::new(thread_counts.async_compute_threads)
            .with_thread_budget_source(TextRasterThreadBudgetSource::TaskPoolAsyncCompute)
    }

    pub(crate) fn with_queue_depth(mut self, queue_depth: usize) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    pub(crate) fn with_thread_budget_source(
        mut self,
        thread_budget_source: TextRasterThreadBudgetSource,
    ) -> Self {
        self.thread_budget_source = thread_budget_source;
        self
    }

    fn normalized(mut self) -> Self {
        self.worker_count = self.worker_count.max(1);
        self
    }
}

impl Default for TextRasterWorkerPoolDiagnostics {
    fn default() -> Self {
        Self {
            thread_budget_source: TextRasterThreadBudgetSource::Explicit,
            budgeted_threads: 0,
            in_flight: 0,
            completed: 0,
            failed: 0,
            queue_peak: 0,
        }
    }
}

impl TextRasterWorkerPoolDiagnostics {
    fn for_options(options: &TextRasterWorkerPoolOptions) -> Self {
        Self {
            thread_budget_source: options.thread_budget_source,
            budgeted_threads: options.worker_count,
            ..Self::default()
        }
    }
}

impl TextRasterWorkerPoolFrameDiagnostics {
    pub(crate) fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        store.record(
            TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC,
            frame_index,
            self.in_flight as f64,
            Some("glyph"),
            ["text", "raster", "worker"],
        );
        store.record(
            TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
            frame_index,
            self.budgeted_threads as f64,
            Some("thread"),
            [
                "text",
                "raster",
                "worker",
                "budget",
                self.thread_budget_source.as_str(),
            ],
        );
        store.record(
            TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
            frame_index,
            self.completed_delta as f64,
            Some("glyph"),
            ["text", "raster", "worker", "frame"],
        );
        store.record(
            TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC,
            frame_index,
            self.failed_delta as f64,
            Some("glyph"),
            ["text", "raster", "worker", "frame"],
        );
    }
}

impl TextRasterWorkerPoolFrameSampler {
    pub(crate) fn from_pool(pool: &TextRasterWorkerPool) -> Self {
        let diagnostics = pool.diagnostics();
        Self {
            last_completed: diagnostics.completed,
            last_failed: diagnostics.failed,
        }
    }

    pub(crate) fn sample(
        &mut self,
        pool: &TextRasterWorkerPool,
    ) -> TextRasterWorkerPoolFrameDiagnostics {
        let diagnostics = pool.diagnostics();
        let frame = TextRasterWorkerPoolFrameDiagnostics {
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

    pub(crate) fn record_diagnostics(
        &mut self,
        pool: &TextRasterWorkerPool,
        store: &mut DiagnosticStore,
        frame_index: u64,
    ) {
        self.sample(pool).record_diagnostics(store, frame_index);
    }
}

impl TextRasterWorkerPool {
    pub(crate) fn new(options: TextRasterWorkerPoolOptions) -> CoreResult<Self> {
        let options = options.normalized();
        let worker_count = options.worker_count;
        let (request_tx, request_rx) = request_channel(options.queue_depth);
        let (completion_tx, completion_rx) = unbounded();
        let in_flight = Arc::new(Mutex::new(HashSet::new()));
        let diagnostics = Arc::new(Mutex::new(TextRasterWorkerPoolDiagnostics::for_options(
            &options,
        )));
        let mut joins = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let request_rx = request_rx.clone();
            let completion_tx = completion_tx.clone();
            let in_flight = Arc::clone(&in_flight);
            let diagnostics = Arc::clone(&diagnostics);
            joins.push(spawn_named_thread(
                format!("zircon-text-raster-{worker_index}"),
                move || {
                    let mut rasterizer = SwashRasterizer::new();
                    while let Ok(work) = request_rx.recv() {
                        let result = TextRasterWorkResult {
                            id: work.id,
                            face_epoch: work.face_epoch,
                            result: rasterizer.rasterize(work.font_data.as_ref(), work.request),
                        };
                        publish_completion(&completion_tx, &in_flight, &diagnostics, result);
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
    pub(crate) fn new_without_workers_for_test(options: TextRasterWorkerPoolOptions) -> Self {
        let options = options.normalized();
        let (request_tx, request_rx) = request_channel(options.queue_depth);
        let (completion_tx, completion_rx) = unbounded();
        let diagnostics = Arc::new(Mutex::new(TextRasterWorkerPoolDiagnostics::for_options(
            &options,
        )));

        Self {
            options,
            request_tx: Some(request_tx),
            request_rx_guard: Some(request_rx),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
            diagnostics,
            completion_tx,
            completion_rx,
            joins: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn request_channel_guard_is_alive_for_test(&self) -> bool {
        self.request_rx_guard.is_some()
    }

    #[cfg(test)]
    pub(crate) fn disconnect_request_channel_for_test(&mut self) {
        self.request_tx.take();
        self.request_rx_guard.take();
    }

    #[cfg(test)]
    pub(crate) fn try_recv_request_for_test(&self) -> Option<TextRasterWorkItem> {
        self.request_rx_guard
            .as_ref()
            .and_then(|request_rx| request_rx.try_recv().ok())
    }

    pub(crate) fn options(&self) -> &TextRasterWorkerPoolOptions {
        &self.options
    }

    pub(crate) fn request(&self, work: TextRasterWorkItem) -> CoreResult<()> {
        let Some(request_tx) = self.request_tx.as_ref() else {
            return Err(CoreError::ChannelSend(
                "text raster worker request channel closed".to_string(),
            ));
        };
        let mut in_flight = self.lock_in_flight();
        if !in_flight.insert(work.id) {
            return Err(CoreError::ChannelSend(format!(
                "text raster work already in flight: {:?}",
                work.id
            )));
        }

        let work_id = work.id;
        if let Err(error) = request_tx.try_send(work) {
            in_flight.remove(&work_id);
            self.record_in_flight_locked(&in_flight);
            return Err(match error {
                TrySendError::Full(work) => {
                    CoreError::ChannelSend(format!("text raster work queue full: {:?}", work.id))
                }
                TrySendError::Disconnected(work) => {
                    CoreError::ChannelSend(format!("text raster work dropped: {:?}", work.id))
                }
            });
        }
        self.record_in_flight_locked(&in_flight);
        Ok(())
    }

    pub(crate) fn completion_receiver(&self) -> ChannelReceiver<TextRasterWorkResult> {
        self.completion_rx.clone()
    }

    pub(crate) fn drain_completed_for_face_epoch(
        &self,
        live_face_epoch: u64,
    ) -> TextRasterCompletionDrain {
        let mut drain = TextRasterCompletionDrain::default();
        while let Ok(result) = self.completion_rx.try_recv() {
            match result.disposition_for_face_epoch(live_face_epoch) {
                TextRasterWorkDisposition::Accepted => drain.accepted.push(result),
                TextRasterWorkDisposition::InvalidatedFace => {
                    drain.face_invalidated_count += 1;
                    drain.face_invalidated_ids.push(result.id);
                }
            }
        }
        drain
    }

    pub(crate) fn diagnostics(&self) -> TextRasterWorkerPoolDiagnostics {
        *self.lock_diagnostics()
    }

    pub(crate) fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        let diagnostics = self.diagnostics();
        for (path, value) in [
            (
                TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC,
                diagnostics.in_flight as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC,
                diagnostics.completed as f64,
            ),
            (
                TEXT_RASTER_WORKER_FAILED_DIAGNOSTIC,
                diagnostics.failed as f64,
            ),
            (
                TEXT_RASTER_WORKER_QUEUE_PEAK_DIAGNOSTIC,
                diagnostics.queue_peak as f64,
            ),
        ] {
            store.record(
                path,
                frame_index,
                value,
                Some("glyph"),
                ["text", "raster", "worker"],
            );
        }
        store.record(
            TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
            frame_index,
            diagnostics.budgeted_threads as f64,
            Some("thread"),
            [
                "text",
                "raster",
                "worker",
                "budget",
                diagnostics.thread_budget_source.as_str(),
            ],
        );
    }

    #[cfg(test)]
    pub(crate) fn publish_completion_for_test(&self, result: TextRasterWorkResult) {
        publish_completion(
            &self.completion_tx,
            &self.in_flight,
            &self.diagnostics,
            result,
        );
    }

    fn record_in_flight_locked(&self, in_flight: &HashSet<TextRasterWorkId>) {
        let in_flight_count = in_flight.len();
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = in_flight_count;
        diagnostics.queue_peak = diagnostics.queue_peak.max(in_flight_count);
    }

    fn lock_in_flight(&self) -> MutexGuard<'_, HashSet<TextRasterWorkId>> {
        lock_in_flight_set(&self.in_flight)
    }

    fn lock_diagnostics(&self) -> MutexGuard<'_, TextRasterWorkerPoolDiagnostics> {
        lock_worker_diagnostics(&self.diagnostics)
    }
}

impl Drop for TextRasterWorkerPool {
    fn drop(&mut self) {
        self.request_tx.take();

        for join in self.joins.drain(..) {
            let _ = join.join();
        }
    }
}

fn request_channel(
    queue_depth: Option<usize>,
) -> (
    ChannelSender<TextRasterWorkItem>,
    ChannelReceiver<TextRasterWorkItem>,
) {
    match queue_depth {
        Some(queue_depth) => bounded(queue_depth),
        None => unbounded(),
    }
}

fn publish_completion(
    completion_tx: &ChannelSender<TextRasterWorkResult>,
    in_flight: &Mutex<HashSet<TextRasterWorkId>>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    result: TextRasterWorkResult,
) {
    let failed = result.result.is_err();
    let remaining_work = {
        let mut in_flight = lock_in_flight_set(in_flight);
        in_flight.remove(&result.id);
        in_flight.len()
    };
    {
        let mut diagnostics = lock_worker_diagnostics(diagnostics);
        diagnostics.in_flight = remaining_work;
        diagnostics.completed += 1;
        if failed {
            diagnostics.failed += 1;
        }
    }
    let _ = completion_tx.send(result);
}

fn lock_in_flight_set(
    in_flight: &Mutex<HashSet<TextRasterWorkId>>,
) -> MutexGuard<'_, HashSet<TextRasterWorkId>> {
    in_flight
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_worker_diagnostics(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
) -> MutexGuard<'_, TextRasterWorkerPoolDiagnostics> {
    diagnostics
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
