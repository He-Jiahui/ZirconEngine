use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::runtime::tasks::TaskPool;

use super::AssetWorkerPool;

pub const ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "asset.worker.in_flight";
pub const ASSET_WORKER_COMPLETED_DIAGNOSTIC: &str = "asset.worker.completed";
pub const ASSET_WORKER_FAILED_DIAGNOSTIC: &str = "asset.worker.failed";
pub const ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "asset.worker.queue_peak";
pub const ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC: &str = "asset.worker.budgeted_threads";
pub const ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC: &str = "asset.worker.frame_completed";
pub const ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC: &str = "asset.worker.frame_failed";
pub const ASSET_WORKER_MERGED_DIAGNOSTIC: &str = "asset.worker.merged";
pub const ASSET_WORKER_REJECTED_DIAGNOSTIC: &str = "asset.worker.rejected";
pub const ASSET_WORKER_EXPIRED_DIAGNOSTIC: &str = "asset.worker.expired";
pub const ASSET_WORKER_CANCELLED_DIAGNOSTIC: &str = "asset.worker.cancelled";
pub const ASSET_WORKER_COMPLETION_BYTES_DIAGNOSTIC: &str = "asset.worker.completion_bytes";
pub const ASSET_WORKER_QUEUE_REJECTED_DIAGNOSTIC: &str = "asset.worker.queue_rejected";
pub const ASSET_WORKER_WAITER_REJECTED_DIAGNOSTIC: &str = "asset.worker.waiter_rejected";
pub const ASSET_WORKER_COMPLETION_REJECTED_DIAGNOSTIC: &str = "asset.worker.completion_rejected";
pub const ASSET_WORKER_QUEUE_AGE_TOTAL_MS_DIAGNOSTIC: &str = "asset.worker.queue_age_total_ms";
pub const ASSET_WORKER_QUEUE_AGE_MAX_MS_DIAGNOSTIC: &str = "asset.worker.queue_age_max_ms";
pub const ASSET_WORKER_QUEUE_AGE_SAMPLES_DIAGNOSTIC: &str = "asset.worker.queue_age_samples";
pub const ASSET_WORKER_PAYLOAD_CLONE_BYTES_DIAGNOSTIC: &str = "asset.worker.payload_clone_bytes";
pub const ASSET_WORKER_CANCEL_WALL_TOTAL_MS_DIAGNOSTIC: &str = "asset.worker.cancel_wall_total_ms";
pub const ASSET_WORKER_CANCEL_WALL_MAX_MS_DIAGNOSTIC: &str = "asset.worker.cancel_wall_max_ms";
pub const ASSET_WORKER_CANCEL_WALL_SAMPLES_DIAGNOSTIC: &str = "asset.worker.cancel_wall_samples";
pub const ASSET_WORKER_DROP_WALL_TOTAL_MS_DIAGNOSTIC: &str = "asset.worker.drop_wall_total_ms";
pub const ASSET_WORKER_DROP_WALL_MAX_MS_DIAGNOSTIC: &str = "asset.worker.drop_wall_max_ms";
pub const ASSET_WORKER_DROP_WALL_SAMPLES_DIAGNOSTIC: &str = "asset.worker.drop_wall_samples";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetWorkerPoolDiagnostics {
    pub thread_budget_source: AssetWorkerThreadBudgetSource,
    pub budgeted_threads: usize,
    pub in_flight: usize,
    pub in_flight_waiters: usize,
    pub completed: u64,
    pub failed: u64,
    pub merged: u64,
    pub rejected: u64,
    pub queue_rejected: u64,
    pub waiter_rejected: u64,
    pub completion_rejected: u64,
    pub expired: u64,
    pub cancelled: u64,
    pub completion_entries: usize,
    pub completion_bytes: usize,
    pub queue_peak: usize,
    /// Byte count copied from a completed payload into observer results. This stays zero because
    /// tickets only clone the result `Arc`, never its CPU payload.
    pub payload_clone_bytes: u64,
    pub queue_age_total: Duration,
    pub queue_age_max: Duration,
    pub queue_age_samples: u64,
    pub cancel_wall_total: Duration,
    pub cancel_wall_max: Duration,
    pub cancel_wall_samples: u64,
    pub drop_wall_total: Duration,
    pub drop_wall_max: Duration,
    pub drop_wall_samples: u64,
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

impl AssetWorkerPoolDiagnostics {
    pub(super) fn for_task_pool(task_pool: &TaskPool) -> Self {
        Self {
            thread_budget_source: AssetWorkerThreadBudgetSource::TaskPoolIo,
            budgeted_threads: task_pool.parallelism(),
            in_flight: 0,
            in_flight_waiters: 0,
            completed: 0,
            failed: 0,
            merged: 0,
            rejected: 0,
            queue_rejected: 0,
            waiter_rejected: 0,
            completion_rejected: 0,
            expired: 0,
            cancelled: 0,
            completion_entries: 0,
            completion_bytes: 0,
            queue_peak: 0,
            payload_clone_bytes: 0,
            queue_age_total: Duration::ZERO,
            queue_age_max: Duration::ZERO,
            queue_age_samples: 0,
            cancel_wall_total: Duration::ZERO,
            cancel_wall_max: Duration::ZERO,
            cancel_wall_samples: 0,
            drop_wall_total: Duration::ZERO,
            drop_wall_max: Duration::ZERO,
            drop_wall_samples: 0,
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
        pool.maintain();
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
    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        let diagnostics = self.diagnostics();
        for (path, value, unit) in [
            (
                ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
                diagnostics.in_flight as f64,
                "request",
            ),
            (
                ASSET_WORKER_COMPLETED_DIAGNOSTIC,
                diagnostics.completed as f64,
                "request",
            ),
            (
                ASSET_WORKER_FAILED_DIAGNOSTIC,
                diagnostics.failed as f64,
                "request",
            ),
            (
                ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
                diagnostics.queue_peak as f64,
                "request",
            ),
            (
                ASSET_WORKER_MERGED_DIAGNOSTIC,
                diagnostics.merged as f64,
                "request",
            ),
            (
                ASSET_WORKER_REJECTED_DIAGNOSTIC,
                diagnostics.rejected as f64,
                "request",
            ),
            (
                ASSET_WORKER_EXPIRED_DIAGNOSTIC,
                diagnostics.expired as f64,
                "request",
            ),
            (
                ASSET_WORKER_CANCELLED_DIAGNOSTIC,
                diagnostics.cancelled as f64,
                "request",
            ),
            (
                ASSET_WORKER_COMPLETION_BYTES_DIAGNOSTIC,
                diagnostics.completion_bytes as f64,
                "byte",
            ),
            (
                ASSET_WORKER_QUEUE_REJECTED_DIAGNOSTIC,
                diagnostics.queue_rejected as f64,
                "request",
            ),
            (
                ASSET_WORKER_WAITER_REJECTED_DIAGNOSTIC,
                diagnostics.waiter_rejected as f64,
                "request",
            ),
            (
                ASSET_WORKER_COMPLETION_REJECTED_DIAGNOSTIC,
                diagnostics.completion_rejected as f64,
                "request",
            ),
            (
                ASSET_WORKER_QUEUE_AGE_TOTAL_MS_DIAGNOSTIC,
                milliseconds(diagnostics.queue_age_total),
                "millisecond",
            ),
            (
                ASSET_WORKER_QUEUE_AGE_MAX_MS_DIAGNOSTIC,
                milliseconds(diagnostics.queue_age_max),
                "millisecond",
            ),
            (
                ASSET_WORKER_QUEUE_AGE_SAMPLES_DIAGNOSTIC,
                diagnostics.queue_age_samples as f64,
                "sample",
            ),
            (
                ASSET_WORKER_PAYLOAD_CLONE_BYTES_DIAGNOSTIC,
                diagnostics.payload_clone_bytes as f64,
                "byte",
            ),
            (
                ASSET_WORKER_CANCEL_WALL_TOTAL_MS_DIAGNOSTIC,
                milliseconds(diagnostics.cancel_wall_total),
                "millisecond",
            ),
            (
                ASSET_WORKER_CANCEL_WALL_MAX_MS_DIAGNOSTIC,
                milliseconds(diagnostics.cancel_wall_max),
                "millisecond",
            ),
            (
                ASSET_WORKER_CANCEL_WALL_SAMPLES_DIAGNOSTIC,
                diagnostics.cancel_wall_samples as f64,
                "sample",
            ),
            (
                ASSET_WORKER_DROP_WALL_TOTAL_MS_DIAGNOSTIC,
                milliseconds(diagnostics.drop_wall_total),
                "millisecond",
            ),
            (
                ASSET_WORKER_DROP_WALL_MAX_MS_DIAGNOSTIC,
                milliseconds(diagnostics.drop_wall_max),
                "millisecond",
            ),
            (
                ASSET_WORKER_DROP_WALL_SAMPLES_DIAGNOSTIC,
                diagnostics.drop_wall_samples as f64,
                "sample",
            ),
        ] {
            store.record(path, frame_index, value, Some(unit), ["asset", "worker"]);
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
}

pub(super) fn record_duration_measurement(
    total: Duration,
    maximum: Duration,
    samples: u64,
    measurement: Duration,
) -> (Duration, Duration, u64) {
    (
        total.saturating_add(measurement),
        maximum.max(measurement),
        samples.saturating_add(1),
    )
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
