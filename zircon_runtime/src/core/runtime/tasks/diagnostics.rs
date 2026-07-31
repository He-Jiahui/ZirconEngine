use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Mutex, MutexGuard,
};
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;

use super::JobSchedulerReport;

pub const TASKS_SCHEDULED_DIAGNOSTIC: &str = "tasks.scheduled";
pub const TASKS_COMPLETED_DIAGNOSTIC: &str = "tasks.completed";
pub const TASKS_DEPENDENCY_WAITING_DIAGNOSTIC: &str = "tasks.dependency_waiting";
pub const TASKS_QUEUED_DIAGNOSTIC: &str = "tasks.queued";
pub const TASKS_ACTIVE_DIAGNOSTIC: &str = "tasks.active";
pub const TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC: &str = "tasks.queue_wait_samples";
pub const TASKS_QUEUE_WAIT_MS_DIAGNOSTIC: &str = "tasks.queue_wait_ms";
pub const TASKS_PANICKED_DIAGNOSTIC: &str = "tasks.panicked";
pub const TASKS_CANCELLED_DIAGNOSTIC: &str = "tasks.cancelled";
pub const TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC: &str = "tasks.dependency_wait_ms";
pub const TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC: &str = "tasks.explicit_wait_ms";

const DIAGNOSTIC_SHARD_COUNT: usize = 64;
const MAX_AGGREGATE_SNAPSHOT_ATTEMPTS: usize = 16;
const UNASSIGNED_SHARD: usize = usize::MAX;

static NEXT_THREAD_SHARD: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static CURRENT_THREAD_SHARD: Cell<usize> = const { Cell::new(UNASSIGNED_SHARD) };
}

pub(super) struct JobSchedulerDiagnosticsState {
    enabled: AtomicBool,
    shards: [DiagnosticsShard; DIAGNOSTIC_SHARD_COUNT],
    last_stable_snapshot: Mutex<JobDiagnosticsSnapshot>,
}

impl Default for JobSchedulerDiagnosticsState {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            shards: std::array::from_fn(|_| DiagnosticsShard::default()),
            last_stable_snapshot: Mutex::new(JobDiagnosticsSnapshot::default()),
        }
    }
}

impl JobSchedulerDiagnosticsState {
    pub(super) fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    pub(super) fn record_scheduled(&self) -> bool {
        let Some(shard) = self.enabled_shard() else {
            return false;
        };
        let _update = shard.begin_update();
        shard.scheduled.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub(super) fn record_scheduled_and_enqueued(&self) -> Option<Instant> {
        let shard = self.enabled_shard()?;
        let _update = shard.begin_update();
        shard.scheduled.fetch_add(1, Ordering::Relaxed);
        shard.enqueued.fetch_add(1, Ordering::Relaxed);
        Some(Instant::now())
    }

    pub(super) fn record_enqueued(&self, tracked: bool) -> Option<Instant> {
        if !tracked {
            return None;
        }
        let shard = self.current_shard();
        let _update = shard.begin_update();
        shard.enqueued.fetch_add(1, Ordering::Relaxed);
        Some(Instant::now())
    }

    pub(super) fn record_started(&self, enqueued_at: Option<Instant>) -> bool {
        let Some(enqueued_at) = enqueued_at else {
            return false;
        };
        let shard = self.current_shard();
        let _update = shard.begin_update();
        add_duration_ns(&shard.queue_wait_ns, enqueued_at.elapsed());
        shard.started.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub(super) fn record_active_terminal(&self, panicked: bool, tracked: bool) {
        if !tracked {
            return;
        }
        let shard = self.current_shard();
        let _update = shard.begin_update();
        if panicked {
            shard.panicked.fetch_add(1, Ordering::Relaxed);
        } else {
            shard.succeeded.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) fn record_cancelled(&self, tracked: bool) {
        if !tracked {
            return;
        }
        let shard = self.current_shard();
        let _update = shard.begin_update();
        shard.cancelled.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_dependency_wait(&self, created_at: Option<Instant>) {
        let Some(created_at) = created_at else {
            return;
        };
        let shard = self.current_shard();
        let _update = shard.begin_update();
        add_duration_ns(&shard.dependency_wait_ns, created_at.elapsed());
    }

    pub(super) fn explicit_wait_started_at(&self) -> Option<Instant> {
        self.enabled.load(Ordering::Relaxed).then(Instant::now)
    }

    pub(super) fn record_explicit_wait(&self, started_at: Option<Instant>) {
        let Some(started_at) = started_at else {
            return;
        };
        let shard = self.current_shard();
        let _update = shard.begin_update();
        add_duration_ns(&shard.explicit_wait_ns, started_at.elapsed());
    }

    pub(super) fn report(&self) -> JobSchedulerReport {
        if !self.enabled.load(Ordering::Acquire) {
            return JobSchedulerReport::default();
        }

        if let Some(snapshot) = self.try_stable_snapshot() {
            let mut cached = self.lock_last_stable_snapshot();
            *cached = snapshot;
            return cached.report();
        }

        self.lock_last_stable_snapshot().report()
    }

    fn enabled_shard(&self) -> Option<&DiagnosticsShard> {
        self.enabled
            .load(Ordering::Relaxed)
            .then(|| self.current_shard())
    }

    fn current_shard(&self) -> &DiagnosticsShard {
        let index = CURRENT_THREAD_SHARD.with(|current| {
            let index = current.get();
            if index != UNASSIGNED_SHARD {
                return index;
            }

            let index = NEXT_THREAD_SHARD.fetch_add(1, Ordering::Relaxed) % DIAGNOSTIC_SHARD_COUNT;
            current.set(index);
            index
        });
        &self.shards[index]
    }

    fn try_stable_snapshot(&self) -> Option<JobDiagnosticsSnapshot> {
        for _ in 0..MAX_AGGREGATE_SNAPSHOT_ATTEMPTS {
            let mut total = JobDiagnosticsSnapshot::default();
            let mut epochs = [0; DIAGNOSTIC_SHARD_COUNT];

            for (index, shard) in self.shards.iter().enumerate() {
                let snapshot = shard.try_stable_snapshot()?;
                epochs[index] = snapshot.epoch;
                total.merge(snapshot.diagnostics);
            }

            // A lifecycle transition can move work from a submitting shard to a worker shard.
            // Recheck every shard after the merge so the aggregate is not assembled across that
            // transition. This stays on the reporting path; writers remain shard-local.
            if self.shards.iter().zip(epochs).all(|(shard, epoch)| {
                shard.updates_in_flight.load(Ordering::Acquire) == 0
                    && shard.update_epoch.load(Ordering::Acquire) == epoch
            }) {
                return Some(total);
            }

            std::hint::spin_loop();
        }

        None
    }

    fn lock_last_stable_snapshot(&self) -> MutexGuard<'_, JobDiagnosticsSnapshot> {
        self.last_stable_snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.report().record_diagnostics(store, frame_index);
    }
}

// A scheduler writer touches one cache-aligned shard. Reporting verifies the merged shard set.
#[repr(align(64))]
struct DiagnosticsShard {
    updates_in_flight: AtomicU64,
    update_epoch: AtomicU64,
    scheduled: AtomicU64,
    enqueued: AtomicU64,
    started: AtomicU64,
    succeeded: AtomicU64,
    panicked: AtomicU64,
    cancelled: AtomicU64,
    queue_wait_ns: AtomicU64,
    dependency_wait_ns: AtomicU64,
    explicit_wait_ns: AtomicU64,
}

impl Default for DiagnosticsShard {
    fn default() -> Self {
        Self {
            updates_in_flight: AtomicU64::new(0),
            update_epoch: AtomicU64::new(0),
            scheduled: AtomicU64::new(0),
            enqueued: AtomicU64::new(0),
            started: AtomicU64::new(0),
            succeeded: AtomicU64::new(0),
            panicked: AtomicU64::new(0),
            cancelled: AtomicU64::new(0),
            queue_wait_ns: AtomicU64::new(0),
            dependency_wait_ns: AtomicU64::new(0),
            explicit_wait_ns: AtomicU64::new(0),
        }
    }
}

impl DiagnosticsShard {
    fn begin_update(&self) -> DiagnosticsUpdate<'_> {
        self.updates_in_flight.fetch_add(1, Ordering::AcqRel);
        self.update_epoch.fetch_add(1, Ordering::Release);
        DiagnosticsUpdate { shard: self }
    }

    fn try_stable_snapshot(&self) -> Option<StableDiagnosticsSnapshot> {
        const MAX_SHARD_SNAPSHOT_ATTEMPTS: usize = 16;

        for _ in 0..MAX_SHARD_SNAPSHOT_ATTEMPTS {
            let epoch_before = self.update_epoch.load(Ordering::Acquire);
            if self.updates_in_flight.load(Ordering::Acquire) != 0 {
                std::hint::spin_loop();
                continue;
            }

            let diagnostics = self.diagnostics_snapshot();
            if self.updates_in_flight.load(Ordering::Acquire) != 0 {
                std::hint::spin_loop();
                continue;
            }
            let epoch_after = self.update_epoch.load(Ordering::Acquire);
            if epoch_before == epoch_after {
                return Some(StableDiagnosticsSnapshot {
                    epoch: epoch_after,
                    diagnostics,
                });
            }
            std::hint::spin_loop();
        }

        None
    }

    fn diagnostics_snapshot(&self) -> JobDiagnosticsSnapshot {
        JobDiagnosticsSnapshot {
            lifecycle: JobLifecycleSnapshot {
                scheduled: self.scheduled.load(Ordering::Relaxed),
                enqueued: self.enqueued.load(Ordering::Relaxed),
                started: self.started.load(Ordering::Relaxed),
                succeeded: self.succeeded.load(Ordering::Relaxed),
                panicked: self.panicked.load(Ordering::Relaxed),
                cancelled: self.cancelled.load(Ordering::Relaxed),
            },
            queue_wait_ns: self.queue_wait_ns.load(Ordering::Relaxed),
            dependency_wait_ns: self.dependency_wait_ns.load(Ordering::Relaxed),
            explicit_wait_ns: self.explicit_wait_ns.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct StableDiagnosticsSnapshot {
    epoch: u64,
    diagnostics: JobDiagnosticsSnapshot,
}

struct DiagnosticsUpdate<'a> {
    shard: &'a DiagnosticsShard,
}

impl Drop for DiagnosticsUpdate<'_> {
    fn drop(&mut self) {
        self.shard.update_epoch.fetch_add(1, Ordering::Release);
        // The retirement chain makes the shard payload visible to a reader that observes 1 -> 0.
        self.shard.updates_in_flight.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct JobDiagnosticsSnapshot {
    lifecycle: JobLifecycleSnapshot,
    queue_wait_ns: u64,
    dependency_wait_ns: u64,
    explicit_wait_ns: u64,
}

impl JobDiagnosticsSnapshot {
    fn merge(&mut self, other: Self) {
        self.lifecycle.merge(other.lifecycle);
        self.queue_wait_ns = self.queue_wait_ns.saturating_add(other.queue_wait_ns);
        self.dependency_wait_ns = self
            .dependency_wait_ns
            .saturating_add(other.dependency_wait_ns);
        self.explicit_wait_ns = self.explicit_wait_ns.saturating_add(other.explicit_wait_ns);
    }

    fn report(self) -> JobSchedulerReport {
        let lifecycle = self.lifecycle;
        JobSchedulerReport {
            scheduled: lifecycle.scheduled,
            completed: lifecycle.completed(),
            dependency_waiting: lifecycle.dependency_waiting(),
            queued: lifecycle.queued(),
            active: lifecycle.active(),
            queue_wait_samples: lifecycle.started,
            queue_wait_ms: duration_ms(self.queue_wait_ns),
            panicked: lifecycle.panicked,
            cancelled: lifecycle.cancelled,
            dependency_wait_ms: duration_ms(self.dependency_wait_ns),
            explicit_wait_ms: duration_ms(self.explicit_wait_ns),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct JobLifecycleSnapshot {
    scheduled: u64,
    enqueued: u64,
    started: u64,
    succeeded: u64,
    panicked: u64,
    cancelled: u64,
}

impl JobLifecycleSnapshot {
    fn merge(&mut self, other: Self) {
        self.scheduled = self.scheduled.saturating_add(other.scheduled);
        self.enqueued = self.enqueued.saturating_add(other.enqueued);
        self.started = self.started.saturating_add(other.started);
        self.succeeded = self.succeeded.saturating_add(other.succeeded);
        self.panicked = self.panicked.saturating_add(other.panicked);
        self.cancelled = self.cancelled.saturating_add(other.cancelled);
    }

    fn completed(self) -> u64 {
        self.succeeded
            .saturating_add(self.panicked)
            .saturating_add(self.cancelled)
    }

    fn queued(self) -> u64 {
        self.enqueued.saturating_sub(self.started)
    }

    fn dependency_waiting(self) -> u64 {
        self.scheduled
            .saturating_sub(self.completed())
            .saturating_sub(self.queued())
            .saturating_sub(self.active())
    }

    fn active(self) -> u64 {
        self.started
            .saturating_sub(self.succeeded.saturating_add(self.panicked))
    }
}

fn add_duration_ns(target: &AtomicU64, elapsed: Duration) {
    let elapsed_ns = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    target.fetch_add(elapsed_ns, Ordering::Relaxed);
}

fn duration_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::JobSchedulerDiagnosticsState;

    #[test]
    fn disabled_diagnostics_do_not_allocate_lifecycle_samples() {
        let state = JobSchedulerDiagnosticsState::default();

        assert!(state.record_scheduled_and_enqueued().is_none());
        assert_eq!(state.report().scheduled, 0);

        state.enable();
        let enqueued_at = state
            .record_scheduled_and_enqueued()
            .expect("enabled diagnostics should record queue admission");
        assert!(state.record_started(Some(enqueued_at)));
        state.record_active_terminal(false, true);

        let report = state.report();
        assert_eq!(report.scheduled, 1);
        assert_eq!(report.completed, 1);
    }

    #[test]
    fn overlapping_diagnostic_writers_publish_one_stable_lifecycle_snapshot() {
        let state = Arc::new(JobSchedulerDiagnosticsState::default());
        state.enable();
        let entered = Arc::new(Barrier::new(3));
        let release = Arc::new(Barrier::new(3));
        let mut writers = Vec::new();

        for _ in 0..2 {
            let state = Arc::clone(&state);
            let entered = Arc::clone(&entered);
            let release = Arc::clone(&release);
            writers.push(thread::spawn(move || {
                let shard = &state.shards[0];
                let _update = shard.begin_update();
                shard
                    .scheduled
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                shard
                    .enqueued
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                entered.wait();
                release.wait();
            }));
        }

        entered.wait();
        release.wait();
        for writer in writers {
            writer.join().unwrap();
        }

        let report = state.report();
        assert_eq!(report.scheduled, 2);
        assert_eq!(report.queued, 2);
        assert_eq!(report.dependency_waiting, 0);

        let source = include_str!("diagnostics.rs");
        assert!(source.contains("const DIAGNOSTIC_SHARD_COUNT: usize = 64"));
        assert!(source.contains("#[repr(align(64))]"));
    }
}
