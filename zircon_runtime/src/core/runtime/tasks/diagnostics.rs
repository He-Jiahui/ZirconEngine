use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;

use super::JobSchedulerReport;

pub const TASKS_SCHEDULED_DIAGNOSTIC: &str = "tasks.scheduled";
pub const TASKS_COMPLETED_DIAGNOSTIC: &str = "tasks.completed";
pub const TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC: &str = "tasks.dependency_wait_ms";
pub const TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC: &str = "tasks.main_thread_wait_ms";

#[derive(Default)]
pub(super) struct JobSchedulerDiagnosticsState {
    scheduled: AtomicU64,
    completed: AtomicU64,
    dependency_wait_ns: AtomicU64,
    main_thread_wait_ns: AtomicU64,
}

impl JobSchedulerDiagnosticsState {
    pub(super) fn record_scheduled(&self) {
        self.scheduled.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_completed(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_dependency_wait(&self, elapsed: Duration) {
        add_duration_ns(&self.dependency_wait_ns, elapsed);
    }

    pub(super) fn record_main_thread_wait(&self, elapsed: Duration) {
        add_duration_ns(&self.main_thread_wait_ns, elapsed);
    }

    pub(super) fn report(&self) -> JobSchedulerReport {
        JobSchedulerReport {
            scheduled: self.scheduled.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
            dependency_wait_ms: duration_ms(self.dependency_wait_ns.load(Ordering::Relaxed)),
            main_thread_wait_ms: duration_ms(self.main_thread_wait_ns.load(Ordering::Relaxed)),
        }
    }

    pub(super) fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.report().record_diagnostics(store, frame_index);
    }
}

fn add_duration_ns(target: &AtomicU64, elapsed: Duration) {
    let elapsed_ns = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    target.fetch_add(elapsed_ns, Ordering::Relaxed);
}

fn duration_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}
