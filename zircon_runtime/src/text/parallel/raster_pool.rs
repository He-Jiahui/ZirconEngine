//! Worker queue for CPU glyph rasterization.

use crossbeam_channel::{TrySendError, bounded};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::channel::{ChannelReceiver, ChannelSender};
use crate::core::runtime::tasks::{TaskPoolOptions, spawn_named_thread};
use crate::core::{CoreError, CoreResult};
use crate::text::raster::{GlyphBitmap, SwashRasterError, SwashRasterRequest, SwashRasterizer};

use super::completion_queue::CompletionByteBudget;

mod worker;

const TEXT_RASTER_WORKER_QUEUE_DEPTH_PER_THREAD: usize = 64;
const TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE: usize = 32;
const TEXT_RASTER_WORKER_COMPLETION_BYTES_PER_THREAD: usize = 2 * 1024 * 1024;

pub(crate) const TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "text.raster.worker.in_flight";
pub(crate) const TEXT_RASTER_WORKER_QUEUED_DIAGNOSTIC: &str = "text.raster.worker.queued";
pub(crate) const TEXT_RASTER_WORKER_RUNNING_DIAGNOSTIC: &str = "text.raster.worker.running";
pub(crate) const TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC: &str = "text.raster.worker.completed";
pub(crate) const TEXT_RASTER_WORKER_FAILED_DIAGNOSTIC: &str = "text.raster.worker.failed";
pub(crate) const TEXT_RASTER_WORKER_CANCELLED_DIAGNOSTIC: &str = "text.raster.worker.cancelled";
pub(crate) const TEXT_RASTER_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "text.raster.worker.queue_peak";
pub(crate) const TEXT_RASTER_WORKER_COMPLETION_BACKLOG_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backlog";
pub(crate) const TEXT_RASTER_WORKER_COMPLETION_BACKLOG_BYTES_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backlog_bytes";
pub(crate) const TEXT_RASTER_WORKER_COMPLETION_BACKPRESSURED_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backpressured";
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
    pub(crate) completion_queue_depth: usize,
    pub(crate) completion_byte_budget: usize,
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolDiagnostics {
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
    pub(crate) budgeted_threads: usize,
    pub(crate) in_flight: usize,
    pub(crate) queued: usize,
    pub(crate) running: usize,
    pub(crate) completed: u64,
    pub(crate) failed: u64,
    pub(crate) cancelled: u64,
    pub(crate) queue_peak: usize,
    pub(crate) completion_backlog: usize,
    pub(crate) completion_backlog_bytes: usize,
    pub(crate) completion_backpressured: u64,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextRasterWorkerRequestError {
    QueueFull(TextRasterWorkId),
    ChannelClosed(TextRasterWorkId),
    DuplicateInFlight(TextRasterWorkId),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextRasterCompletionDrain {
    pub(crate) accepted: Vec<TextRasterWorkResult>,
    pub(crate) face_invalidated_ids: Vec<TextRasterWorkId>,
    pub(crate) face_invalidated_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextRasterCompletionDrainBudget {
    max_items: usize,
    max_bytes: usize,
}

#[derive(Default)]
struct TextRasterWorkerWorkState {
    in_flight: HashSet<TextRasterWorkId>,
    running: HashSet<TextRasterWorkId>,
    cancelled: HashSet<TextRasterWorkId>,
}

pub(crate) struct TextRasterWorkerPool {
    options: TextRasterWorkerPoolOptions,
    request_tx: Option<ChannelSender<TextRasterWorkItem>>,
    #[cfg(test)]
    request_rx_guard: Option<ChannelReceiver<TextRasterWorkItem>>,
    work_state: Arc<Mutex<TextRasterWorkerWorkState>>,
    diagnostics: Arc<Mutex<TextRasterWorkerPoolDiagnostics>>,
    completion_tx: ChannelSender<TextRasterWorkResult>,
    completion_rx: Option<ChannelReceiver<TextRasterWorkResult>>,
    completion_byte_budget: Arc<CompletionByteBudget>,
    shutdown: Arc<AtomicBool>,
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

    fn byte_count(&self) -> usize {
        self.result.as_ref().map_or(0, |bitmap| bitmap.data.len())
    }
}

impl TextRasterCompletionDrainBudget {
    pub(crate) const fn new(max_items: usize, max_bytes: usize) -> Self {
        Self {
            max_items,
            max_bytes,
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
        let worker_count = worker_count.max(1);
        Self {
            worker_count,
            queue_depth: None,
            completion_queue_depth: worker_count
                .saturating_mul(TEXT_RASTER_WORKER_QUEUE_DEPTH_PER_THREAD),
            completion_byte_budget: worker_count
                .saturating_mul(TEXT_RASTER_WORKER_COMPLETION_BYTES_PER_THREAD),
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

    pub(crate) fn with_completion_queue_depth(mut self, completion_queue_depth: usize) -> Self {
        self.completion_queue_depth = completion_queue_depth;
        self
    }

    pub(crate) fn with_completion_byte_budget(mut self, completion_byte_budget: usize) -> Self {
        self.completion_byte_budget = completion_byte_budget;
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
        if self.queue_depth.is_none() {
            self.queue_depth = Some(
                self.worker_count
                    .saturating_mul(TEXT_RASTER_WORKER_QUEUE_DEPTH_PER_THREAD),
            );
        }
        self.completion_queue_depth = self.completion_queue_depth.max(1);
        self.completion_byte_budget = self.completion_byte_budget.max(1);
        self
    }
}

impl Default for TextRasterWorkerPoolDiagnostics {
    fn default() -> Self {
        Self {
            thread_budget_source: TextRasterThreadBudgetSource::Explicit,
            budgeted_threads: 0,
            in_flight: 0,
            queued: 0,
            running: 0,
            completed: 0,
            failed: 0,
            cancelled: 0,
            queue_peak: 0,
            completion_backlog: 0,
            completion_backlog_bytes: 0,
            completion_backpressured: 0,
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
        let (completion_tx, completion_rx) = bounded(options.completion_queue_depth);
        let work_state = Arc::new(Mutex::new(TextRasterWorkerWorkState::default()));
        let diagnostics = Arc::new(Mutex::new(TextRasterWorkerPoolDiagnostics::for_options(
            &options,
        )));
        let completion_byte_budget =
            Arc::new(CompletionByteBudget::new(options.completion_byte_budget));
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut joins = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let request_rx = request_rx.clone();
            let completion_tx = completion_tx.clone();
            let work_state = Arc::clone(&work_state);
            let diagnostics = Arc::clone(&diagnostics);
            let completion_byte_budget = Arc::clone(&completion_byte_budget);
            let shutdown = Arc::clone(&shutdown);
            joins.push(spawn_named_thread(
                format!("zircon-text-raster-{worker_index}"),
                move || {
                    let mut rasterizer = SwashRasterizer::new();
                    while let Ok(work) = request_rx.recv() {
                        if shutdown.load(Ordering::Acquire) {
                            break;
                        }
                        let mut work_batch =
                            Vec::with_capacity(TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE);
                        work_batch.push(work);
                        while work_batch.len() < TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE {
                            let Ok(next_work) = request_rx.try_recv() else {
                                break;
                            };
                            work_batch.push(next_work);
                        }
                        worker::process_worker_batch(
                            &mut rasterizer,
                            &completion_tx,
                            &work_state,
                            &diagnostics,
                            &completion_byte_budget,
                            work_batch,
                        );
                    }
                },
            )?);
        }

        Ok(Self {
            options,
            request_tx: Some(request_tx),
            #[cfg(test)]
            request_rx_guard: None,
            work_state,
            diagnostics,
            completion_tx,
            completion_rx: Some(completion_rx),
            completion_byte_budget,
            shutdown,
            joins,
        })
    }

    pub(crate) fn options(&self) -> &TextRasterWorkerPoolOptions {
        &self.options
    }

    pub(crate) fn request(&self, work: TextRasterWorkItem) -> CoreResult<()> {
        self.try_request(work)
            .map_err(|error| CoreError::ChannelSend(error.to_string()))
    }

    pub(crate) fn try_request(
        &self,
        work: TextRasterWorkItem,
    ) -> Result<(), TextRasterWorkerRequestError> {
        let Some(request_tx) = self.request_tx.as_ref() else {
            return Err(TextRasterWorkerRequestError::ChannelClosed(work.id));
        };
        let mut work_state = self.lock_work_state();
        if !work_state.in_flight.insert(work.id) {
            return Err(TextRasterWorkerRequestError::DuplicateInFlight(work.id));
        }

        let work_id = work.id;
        if let Err(error) = request_tx.try_send(work) {
            work_state.in_flight.remove(&work_id);
            work_state.cancelled.remove(&work_id);
            self.record_in_flight_locked(&work_state);
            return Err(match error {
                TrySendError::Full(_) => TextRasterWorkerRequestError::QueueFull(work_id),
                TrySendError::Disconnected(_) => {
                    TextRasterWorkerRequestError::ChannelClosed(work_id)
                }
            });
        }
        self.record_in_flight_locked(&work_state);
        Ok(())
    }

    pub(crate) fn cancel(&self, work_id: TextRasterWorkId) -> bool {
        let mut work_state = self.lock_work_state();
        if !work_state.in_flight.contains(&work_id) {
            return false;
        }
        work_state.cancelled.insert(work_id)
    }

    pub(crate) fn cancel_all(&self) -> usize {
        let mut work_state = self.lock_work_state();
        let pending_ids = work_state.in_flight.iter().copied().collect::<Vec<_>>();
        let mut cancelled_count = 0;
        for work_id in pending_ids {
            if work_state.cancelled.insert(work_id) {
                cancelled_count += 1;
            }
        }
        cancelled_count
    }

    pub(crate) fn drain_completed_for_face_epoch(
        &self,
        live_face_epoch: u64,
        budget: TextRasterCompletionDrainBudget,
    ) -> TextRasterCompletionDrain {
        let mut drain = TextRasterCompletionDrain::default();
        let Some(completion_rx) = self.completion_rx.as_ref() else {
            return drain;
        };

        let mut drained_bytes = 0;
        while drain
            .accepted
            .len()
            .saturating_add(drain.face_invalidated_count)
            < budget.max_items
            && drained_bytes < budget.max_bytes
        {
            let Ok(result) = completion_rx.try_recv() else {
                break;
            };
            let result_bytes = result.byte_count();
            self.release_completion_backlog(result_bytes);
            drained_bytes = drained_bytes.saturating_add(result_bytes);
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
                TEXT_RASTER_WORKER_QUEUED_DIAGNOSTIC,
                diagnostics.queued as f64,
            ),
            (
                TEXT_RASTER_WORKER_RUNNING_DIAGNOSTIC,
                diagnostics.running as f64,
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
                TEXT_RASTER_WORKER_CANCELLED_DIAGNOSTIC,
                diagnostics.cancelled as f64,
            ),
            (
                TEXT_RASTER_WORKER_QUEUE_PEAK_DIAGNOSTIC,
                diagnostics.queue_peak as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_BACKLOG_DIAGNOSTIC,
                diagnostics.completion_backlog as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_BACKLOG_BYTES_DIAGNOSTIC,
                diagnostics.completion_backlog_bytes as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_BACKPRESSURED_DIAGNOSTIC,
                diagnostics.completion_backpressured as f64,
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
        assert!(self.try_publish_completion_for_test(result));
    }

    #[cfg(test)]
    pub(crate) fn try_publish_completion_for_test(&self, result: TextRasterWorkResult) -> bool {
        let result_bytes = result.byte_count();
        if !self.completion_byte_budget.try_reserve(result_bytes) {
            self.record_completion_backpressured();
            return false;
        }
        match self.completion_tx.try_send(result) {
            Ok(()) => {
                self.record_completion_backlog(result_bytes);
                self.record_test_completion(result_bytes > 0, false);
                true
            }
            Err(_) => {
                self.completion_byte_budget.release(result_bytes);
                self.record_completion_backpressured();
                false
            }
        }
    }

    fn record_in_flight_locked(&self, work_state: &TextRasterWorkerWorkState) {
        let in_flight_count = work_state.in_flight.len();
        let running_count = work_state.running.len();
        let queued_count = in_flight_count.saturating_sub(running_count);
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = in_flight_count;
        diagnostics.queued = queued_count;
        diagnostics.running = running_count;
        diagnostics.queue_peak = diagnostics.queue_peak.max(queued_count);
    }

    fn record_completion_backlog(&self, result_bytes: usize) {
        worker::record_completion_backlog(&self.diagnostics, result_bytes);
    }

    fn release_completion_backlog(&self, result_bytes: usize) {
        self.completion_byte_budget.release(result_bytes);
        worker::release_completion_backlog(&self.diagnostics, result_bytes);
    }

    fn record_completion_backpressured(&self) {
        worker::record_completion_backpressured(&self.diagnostics);
    }

    #[cfg(test)]
    fn record_test_completion(&self, succeeded: bool, cancelled: bool) {
        let mut diagnostics = self.lock_diagnostics();
        if cancelled {
            diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
            return;
        }
        diagnostics.completed = diagnostics.completed.saturating_add(1);
        if !succeeded {
            diagnostics.failed = diagnostics.failed.saturating_add(1);
        }
    }

    fn lock_work_state(&self) -> MutexGuard<'_, TextRasterWorkerWorkState> {
        worker::lock_worker_work_state(&self.work_state)
    }

    fn lock_diagnostics(&self) -> MutexGuard<'_, TextRasterWorkerPoolDiagnostics> {
        worker::lock_worker_diagnostics(&self.diagnostics)
    }
}

impl std::fmt::Display for TextRasterWorkerRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull(work_id) => {
                write!(formatter, "text raster work queue full: {work_id:?}")
            }
            Self::ChannelClosed(work_id) => {
                write!(
                    formatter,
                    "text raster worker request channel closed: {work_id:?}"
                )
            }
            Self::DuplicateInFlight(work_id) => {
                write!(formatter, "text raster work already in flight: {work_id:?}")
            }
        }
    }
}

impl Drop for TextRasterWorkerPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.cancel_all();
        self.request_tx.take();
        self.completion_byte_budget.close();
        self.completion_rx.take();

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
    bounded(queue_depth.unwrap_or(1))
}
