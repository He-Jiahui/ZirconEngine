use crate::core::diagnostics::DiagnosticStore;

use super::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};

pub(crate) const TEXT_RASTER_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "text.raster.worker.in_flight";
pub(crate) const TEXT_RASTER_WORKER_QUEUED_DIAGNOSTIC: &str = "text.raster.worker.queued";
const TEXT_RASTER_WORKER_QUEUED_INPUT_BYTES_DIAGNOSTIC: &str =
    "text.raster.worker.queued_input_bytes";
pub(crate) const TEXT_RASTER_WORKER_RUNNING_DIAGNOSTIC: &str = "text.raster.worker.running";
pub(crate) const TEXT_RASTER_WORKER_COMPLETED_DIAGNOSTIC: &str = "text.raster.worker.completed";
const TEXT_RASTER_WORKER_FAILED_DIAGNOSTIC: &str = "text.raster.worker.failed";
const TEXT_RASTER_WORKER_CANCELLED_DIAGNOSTIC: &str = "text.raster.worker.cancelled";
const TEXT_RASTER_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "text.raster.worker.queue_peak";
const TEXT_RASTER_WORKER_COMPLETION_BACKLOG_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backlog";
const TEXT_RASTER_WORKER_COMPLETION_BACKLOG_BYTES_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backlog_bytes";
const TEXT_RASTER_WORKER_COMPLETION_BACKPRESSURED_DIAGNOSTIC: &str =
    "text.raster.worker.completion_backpressured";
const TEXT_RASTER_WORKER_COMPLETION_BUDGET_REJECTED_DIAGNOSTIC: &str =
    "text.raster.worker.completion_budget_rejected";
const TEXT_RASTER_WORKER_COMPLETION_REJECTED_BYTES_DIAGNOSTIC: &str =
    "text.raster.worker.completion_rejected_bytes";
const TEXT_RASTER_WORKER_REQUEST_BACKPRESSURED_DIAGNOSTIC: &str =
    "text.raster.worker.request_backpressured";
pub(crate) const TEXT_RASTER_WORKER_BUDGETED_THREADS_DIAGNOSTIC: &str =
    "text.raster.worker.budgeted_threads";
pub(crate) const TEXT_RASTER_WORKER_FRAME_COMPLETED_DIAGNOSTIC: &str =
    "text.raster.worker.frame_completed";
pub(crate) const TEXT_RASTER_WORKER_FRAME_FAILED_DIAGNOSTIC: &str =
    "text.raster.worker.frame_failed";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextRasterThreadBudgetSource {
    #[default]
    Explicit,
    TaskPoolAsyncCompute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextRasterWorkerPoolDiagnostics {
    pub(crate) thread_budget_source: TextRasterThreadBudgetSource,
    pub(crate) budgeted_threads: usize,
    pub(crate) in_flight: usize,
    pub(crate) queued: usize,
    pub(crate) queued_input_bytes: usize,
    pub(crate) running: usize,
    pub(crate) completed: u64,
    pub(crate) failed: u64,
    pub(crate) cancelled: u64,
    pub(crate) queue_peak: usize,
    pub(crate) completion_backlog: usize,
    pub(crate) completion_backlog_bytes: usize,
    pub(crate) completion_backpressured: u64,
    pub(crate) completion_budget_rejected: u64,
    pub(crate) completion_rejected_bytes: u64,
    pub(crate) request_backpressured: u64,
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

impl TextRasterThreadBudgetSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::TaskPoolAsyncCompute => "task_pool_async_compute",
        }
    }
}

impl Default for TextRasterWorkerPoolDiagnostics {
    fn default() -> Self {
        Self {
            thread_budget_source: TextRasterThreadBudgetSource::Explicit,
            budgeted_threads: 0,
            in_flight: 0,
            queued: 0,
            queued_input_bytes: 0,
            running: 0,
            completed: 0,
            failed: 0,
            cancelled: 0,
            queue_peak: 0,
            completion_backlog: 0,
            completion_backlog_bytes: 0,
            completion_backpressured: 0,
            completion_budget_rejected: 0,
            completion_rejected_bytes: 0,
            request_backpressured: 0,
        }
    }
}

impl TextRasterWorkerPoolDiagnostics {
    pub(super) fn for_options(options: &TextRasterWorkerPoolOptions) -> Self {
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
                TEXT_RASTER_WORKER_COMPLETION_BACKPRESSURED_DIAGNOSTIC,
                diagnostics.completion_backpressured as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_BUDGET_REJECTED_DIAGNOSTIC,
                diagnostics.completion_budget_rejected as f64,
            ),
            (
                TEXT_RASTER_WORKER_REQUEST_BACKPRESSURED_DIAGNOSTIC,
                diagnostics.request_backpressured as f64,
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
        for (path, value) in [
            (
                TEXT_RASTER_WORKER_QUEUED_INPUT_BYTES_DIAGNOSTIC,
                diagnostics.queued_input_bytes as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_BACKLOG_BYTES_DIAGNOSTIC,
                diagnostics.completion_backlog_bytes as f64,
            ),
            (
                TEXT_RASTER_WORKER_COMPLETION_REJECTED_BYTES_DIAGNOSTIC,
                diagnostics.completion_rejected_bytes as f64,
            ),
        ] {
            store.record(
                path,
                frame_index,
                value,
                Some("byte"),
                ["text", "raster", "worker", "memory"],
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
}
