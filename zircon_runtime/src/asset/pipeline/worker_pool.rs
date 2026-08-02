//! Runtime IO-pool orchestration for CPU-side asset decoding.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::asset::types::{AssetRequest, CpuAssetPayload};
use crate::core::runtime::tasks::{TaskPool, TaskPoolKind, TaskTimer};
use crate::core::{CoreError, CoreResult};

mod completion;
mod diagnostics;
mod options;
mod payload;

pub use completion::{AssetWorkerCompletionError, AssetWorkerCompletionTicket};
use completion::{
    AssetWorkerExpiryTimer, AssetWorkerRejectionKind, CompletionEntry, CompletionRegistry,
    CompletionTerminal, ExpiryReport, WaiterAdmission, deadline_after, expire_entries,
    lock_completion_registry, lock_worker_diagnostics, maintain_completion_registry,
    publish_completion, record_expiry_for_diagnostics, record_queue_age, schedule_entry_expiry,
};
use diagnostics::record_duration_measurement;
pub use diagnostics::{
    ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC, ASSET_WORKER_CANCEL_WALL_MAX_MS_DIAGNOSTIC,
    ASSET_WORKER_CANCEL_WALL_SAMPLES_DIAGNOSTIC, ASSET_WORKER_CANCEL_WALL_TOTAL_MS_DIAGNOSTIC,
    ASSET_WORKER_CANCELLED_DIAGNOSTIC, ASSET_WORKER_COMPLETED_DIAGNOSTIC,
    ASSET_WORKER_COMPLETION_BYTES_DIAGNOSTIC, ASSET_WORKER_COMPLETION_REJECTED_DIAGNOSTIC,
    ASSET_WORKER_DROP_WALL_MAX_MS_DIAGNOSTIC, ASSET_WORKER_DROP_WALL_SAMPLES_DIAGNOSTIC,
    ASSET_WORKER_DROP_WALL_TOTAL_MS_DIAGNOSTIC, ASSET_WORKER_EXPIRED_DIAGNOSTIC,
    ASSET_WORKER_FAILED_DIAGNOSTIC, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
    ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC, ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
    ASSET_WORKER_MERGED_DIAGNOSTIC, ASSET_WORKER_PAYLOAD_CLONE_BYTES_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_AGE_MAX_MS_DIAGNOSTIC, ASSET_WORKER_QUEUE_AGE_SAMPLES_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_AGE_TOTAL_MS_DIAGNOSTIC, ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC,
    ASSET_WORKER_QUEUE_REJECTED_DIAGNOSTIC, ASSET_WORKER_REJECTED_DIAGNOSTIC,
    ASSET_WORKER_WAITER_REJECTED_DIAGNOSTIC, AssetWorkerPoolDiagnostics,
    AssetWorkerPoolFrameDiagnostics, AssetWorkerPoolFrameSampler, AssetWorkerThreadBudgetSource,
};
pub use options::AssetWorkerPoolOptions;
use options::DEFAULT_ASSET_WORKER_QUEUE_DEPTH;
use payload::process_request;

#[cfg(test)]
#[path = "worker_pool/tests.rs"]
mod tests;

/// Shared owner for one immutable CPU payload. Tickets are observers, never copies.
pub struct AssetWorkerPool {
    options: AssetWorkerPoolOptions,
    task_pool: TaskPool,
    expiry_timer: AssetWorkerExpiryTimer,
    completions: Arc<CompletionRegistry>,
    diagnostics: Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    #[cfg(test)]
    test_execution_gate: Option<tests::AssetWorkerTestExecutionGate>,
}

impl AssetWorkerPool {
    pub fn new(task_pool: TaskPool, options: AssetWorkerPoolOptions) -> Self {
        Self::new_with_expiry_timer(task_pool, options, AssetWorkerExpiryTimer::ProcessDefault)
    }

    /// Creates an asset worker pool bound to an explicit runtime lifecycle timer.
    pub(crate) fn with_expiry_timer(
        task_pool: TaskPool,
        options: AssetWorkerPoolOptions,
        timer: TaskTimer,
    ) -> Self {
        Self::new_with_expiry_timer(task_pool, options, AssetWorkerExpiryTimer::Explicit(timer))
    }

    fn new_with_expiry_timer(
        task_pool: TaskPool,
        options: AssetWorkerPoolOptions,
        expiry_timer: AssetWorkerExpiryTimer,
    ) -> Self {
        assert_eq!(
            task_pool.kind(),
            TaskPoolKind::Io,
            "AssetWorkerPool requires the runtime IO task pool"
        );
        let diagnostics = Arc::new(Mutex::new(AssetWorkerPoolDiagnostics::for_task_pool(
            &task_pool,
        )));
        let completions = Arc::new(CompletionRegistry::new());
        Self {
            options,
            task_pool,
            expiry_timer,
            completions,
            diagnostics,
            #[cfg(test)]
            test_execution_gate: None,
        }
    }

    pub fn options(&self) -> &AssetWorkerPoolOptions {
        &self.options
    }

    pub fn task_pool(&self) -> &TaskPool {
        &self.task_pool
    }

    /// Reaps request and completion entries whose bounded lifetime elapsed.
    ///
    /// Reaps entries whose deadline elapsed before their timer callback acquired the registry.
    pub fn maintain(&self) {
        maintain_completion_registry(&self.completions, &self.diagnostics);
    }

    /// Starts or joins one bounded single-flight request.
    pub fn request(&self, request: AssetRequest) -> CoreResult<AssetWorkerCompletionTicket> {
        let now = Instant::now();
        let mut state = lock_completion_registry(&self.completions);
        let expiry = expire_entries(&mut state, now);
        self.record_expiry(expiry);
        if state.closing {
            return Err(CoreError::ChannelSend(
                "asset worker pool is shutting down".to_string(),
            ));
        }
        if let Some(entry) = state.in_flight.get(&request).cloned() {
            match entry.try_add_waiter(self.options.waiter_capacity) {
                WaiterAdmission::Added => {
                    self.record_merge(true);
                    return Ok(self.completion_ticket(entry));
                }
                WaiterAdmission::Full => {
                    self.record_rejection(AssetWorkerRejectionKind::Waiter);
                    return Err(CoreError::ChannelSend(format!(
                        "asset request observer budget full: {request:?}"
                    )));
                }
                WaiterAdmission::Terminal => {
                    state.in_flight.remove(&request);
                    let waiters = entry.waiter_count();
                    entry.cancel_expiry();
                    self.record_expiry(ExpiryReport {
                        in_flight_entries: 1,
                        in_flight_waiters: waiters,
                        ..ExpiryReport::default()
                    });
                }
            }
        }
        if let Some(entry) = state
            .completed
            .get(&request)
            .map(|completed| Arc::clone(&completed.entry))
        {
            // A retained immutable result is charged by entry and bytes, not by live observers.
            self.record_merge(false);
            return Ok(self.completion_ticket(entry));
        }
        if self.unique_request_capacity_reached(state.scheduled_jobs) {
            self.record_rejection(AssetWorkerRejectionKind::Queue);
            return Err(CoreError::ChannelSend(format!(
                "asset request queue full: {request:?}"
            )));
        }
        if self.options.waiter_capacity == 0 {
            self.record_rejection(AssetWorkerRejectionKind::Waiter);
            return Err(CoreError::ChannelSend(format!(
                "asset request observer budget full: {request:?}"
            )));
        }

        let request_deadline = deadline_after(now, self.options.request_max_age);
        let entry = Arc::new(CompletionEntry::new(request.clone(), request_deadline));
        let expiry_generation = entry.begin_expiry();
        let expiry_subscription = schedule_entry_expiry(
            &self.expiry_timer,
            &self.completions,
            &self.diagnostics,
            &entry,
            request_deadline,
            expiry_generation,
        )?;
        entry.install_expiry_subscription(expiry_generation, expiry_subscription);
        state.in_flight.insert(request.clone(), Arc::clone(&entry));
        state.scheduled_jobs = state.scheduled_jobs.saturating_add(1);
        drop(state);
        self.record_request_admitted();

        let task_pool = self.task_pool.clone();
        let completions = Arc::clone(&self.completions);
        let diagnostics = Arc::clone(&self.diagnostics);
        let options = self.options.clone();
        let expiry_timer = self.expiry_timer.clone();
        let task_entry = Arc::clone(&entry);
        #[cfg(test)]
        let test_execution_gate = self.test_execution_gate.clone();
        task_pool.spawn(move || {
            let panic_request = request.clone();
            let queue_age = task_entry.mark_started(Instant::now());
            #[cfg(test)]
            if queue_age.is_some() {
                if let Some(gate) = test_execution_gate {
                    gate.wait_for_test_release();
                }
            }
            let payload = if let Some(queue_age) = queue_age {
                record_queue_age(&diagnostics, queue_age);
                Some(
                    catch_unwind(AssertUnwindSafe(|| process_request(request))).unwrap_or(
                        CpuAssetPayload::Failure {
                            request: panic_request,
                            message: "asset worker task panicked".to_string(),
                        },
                    ),
                )
            } else {
                None
            };
            publish_completion(
                &expiry_timer,
                &completions,
                &diagnostics,
                &options,
                task_entry,
                payload,
            );
        });
        Ok(self.completion_ticket(entry))
    }

    /// Cancels a queued, running, or unharvested completion without waiting for workers.
    pub fn cancel(&self, request: &AssetRequest) -> bool {
        let cancel_started = Instant::now();
        let mut state = lock_completion_registry(&self.completions);
        let expiry = expire_entries(&mut state, cancel_started);
        self.record_expiry(expiry);
        if let Some(entry) = state.in_flight.remove(request) {
            let waiters = entry.terminate(CompletionTerminal::Cancelled).unwrap_or(0);
            drop(state);
            let mut diagnostics = self.lock_diagnostics();
            diagnostics.in_flight = diagnostics.in_flight.saturating_sub(1);
            diagnostics.in_flight_waiters = diagnostics.in_flight_waiters.saturating_sub(waiters);
            diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
            (
                diagnostics.cancel_wall_total,
                diagnostics.cancel_wall_max,
                diagnostics.cancel_wall_samples,
            ) = record_duration_measurement(
                diagnostics.cancel_wall_total,
                diagnostics.cancel_wall_max,
                diagnostics.cancel_wall_samples,
                cancel_started.elapsed(),
            );
            return true;
        }
        if let Some(completed) = state.completed.remove(request) {
            state.completed_bytes = state.completed_bytes.saturating_sub(completed.bytes);
            let cancelled = completed
                .entry
                .terminate(CompletionTerminal::Cancelled)
                .is_some();
            drop(state);
            let mut diagnostics = self.lock_diagnostics();
            diagnostics.completion_entries = diagnostics.completion_entries.saturating_sub(1);
            diagnostics.completion_bytes =
                diagnostics.completion_bytes.saturating_sub(completed.bytes);
            if cancelled {
                diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
                (
                    diagnostics.cancel_wall_total,
                    diagnostics.cancel_wall_max,
                    diagnostics.cancel_wall_samples,
                ) = record_duration_measurement(
                    diagnostics.cancel_wall_total,
                    diagnostics.cancel_wall_max,
                    diagnostics.cancel_wall_samples,
                    cancel_started.elapsed(),
                );
            }
            return cancelled;
        }
        false
    }

    pub fn diagnostics(&self) -> AssetWorkerPoolDiagnostics {
        *self.lock_diagnostics()
    }

    fn unique_request_capacity_reached(&self, unique_in_flight: usize) -> bool {
        let queue_depth = self
            .options
            .queue_depth
            .unwrap_or(DEFAULT_ASSET_WORKER_QUEUE_DEPTH);
        let capacity = self.task_pool.parallelism().saturating_add(queue_depth);
        unique_in_flight >= capacity
    }

    fn record_request_admitted(&self) {
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = diagnostics.in_flight.saturating_add(1);
        diagnostics.in_flight_waiters = diagnostics.in_flight_waiters.saturating_add(1);
        diagnostics.queue_peak = diagnostics.queue_peak.max(diagnostics.in_flight);
    }

    fn completion_ticket(&self, entry: Arc<CompletionEntry>) -> AssetWorkerCompletionTicket {
        AssetWorkerCompletionTicket::new(
            entry,
            Arc::clone(&self.completions),
            Arc::clone(&self.diagnostics),
        )
    }

    fn record_merge(&self, in_flight: bool) {
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.merged = diagnostics.merged.saturating_add(1);
        if in_flight {
            diagnostics.in_flight_waiters = diagnostics.in_flight_waiters.saturating_add(1);
        }
    }

    fn record_rejection(&self, kind: AssetWorkerRejectionKind) {
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.rejected = diagnostics.rejected.saturating_add(1);
        match kind {
            AssetWorkerRejectionKind::Queue => {
                diagnostics.queue_rejected = diagnostics.queue_rejected.saturating_add(1);
            }
            AssetWorkerRejectionKind::Waiter => {
                diagnostics.waiter_rejected = diagnostics.waiter_rejected.saturating_add(1);
            }
        }
    }

    fn record_expiry(&self, expiry: ExpiryReport) {
        if expiry.is_empty() {
            return;
        }
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = diagnostics
            .in_flight
            .saturating_sub(expiry.in_flight_entries);
        diagnostics.in_flight_waiters = diagnostics
            .in_flight_waiters
            .saturating_sub(expiry.in_flight_waiters);
        diagnostics.completion_entries = diagnostics
            .completion_entries
            .saturating_sub(expiry.completed_entries);
        diagnostics.completion_bytes = diagnostics
            .completion_bytes
            .saturating_sub(expiry.completed_bytes);
        diagnostics.expired = diagnostics
            .expired
            .saturating_add(expiry.total_entries() as u64);
    }

    fn lock_diagnostics(&self) -> MutexGuard<'_, AssetWorkerPoolDiagnostics> {
        lock_worker_diagnostics(&self.diagnostics)
    }
}

impl Drop for AssetWorkerPool {
    fn drop(&mut self) {
        let drop_started = Instant::now();
        let mut state = lock_completion_registry(&self.completions);
        state.closing = true;
        let in_flight = std::mem::take(&mut state.in_flight);
        let completed = std::mem::take(&mut state.completed);
        state.completed_bytes = 0;
        drop(state);

        let mut cancelled = 0usize;
        for entry in in_flight.into_values() {
            if entry.terminate(CompletionTerminal::Cancelled).is_some() {
                cancelled = cancelled.saturating_add(1);
            }
        }
        for completed in completed.into_values() {
            if completed
                .entry
                .terminate(CompletionTerminal::Cancelled)
                .is_some()
            {
                cancelled = cancelled.saturating_add(1);
            }
        }
        let mut diagnostics = self.lock_diagnostics();
        diagnostics.in_flight = 0;
        diagnostics.in_flight_waiters = 0;
        diagnostics.completion_entries = 0;
        diagnostics.completion_bytes = 0;
        diagnostics.cancelled = diagnostics.cancelled.saturating_add(cancelled as u64);
        (
            diagnostics.drop_wall_total,
            diagnostics.drop_wall_max,
            diagnostics.drop_wall_samples,
        ) = record_duration_measurement(
            diagnostics.drop_wall_total,
            diagnostics.drop_wall_max,
            diagnostics.drop_wall_samples,
            drop_started.elapsed(),
        );
    }
}
