use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;

use super::diagnostic_observation::TaskDiagnosticJournal;
use super::{JobSchedulerReport, TaskDiagnosticIdentity, TaskDiagnosticKind, TaskDiagnosticSource};

mod snapshot;

use snapshot::{JobDiagnosticsSnapshot, JobLifecycleSnapshot, StableDiagnosticsSnapshot};

pub const TASKS_SCHEDULED_DIAGNOSTIC: &str = "tasks.scheduled";
pub const TASKS_COMPLETED_DIAGNOSTIC: &str = "tasks.completed";
pub const TASKS_DEPENDENCY_WAITING_DIAGNOSTIC: &str = "tasks.dependency_waiting";
pub const TASKS_QUEUED_DIAGNOSTIC: &str = "tasks.queued";
pub const TASKS_ACTIVE_DIAGNOSTIC: &str = "tasks.active";
pub const TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC: &str = "tasks.queue_wait_samples";
pub const TASKS_QUEUE_WAIT_MS_DIAGNOSTIC: &str = "tasks.queue_wait_ms";
pub const TASKS_EXECUTION_SAMPLES_DIAGNOSTIC: &str = "tasks.execution_samples";
pub const TASKS_EXECUTION_MS_DIAGNOSTIC: &str = "tasks.execution_ms";
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
    observation_enabled: AtomicBool,
    shards: [DiagnosticsShard; DIAGNOSTIC_SHARD_COUNT],
    last_stable_snapshot: Mutex<JobDiagnosticsSnapshot>,
    observation_journal: Arc<TaskDiagnosticJournal>,
}

impl Default for JobSchedulerDiagnosticsState {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            observation_enabled: AtomicBool::new(false),
            shards: std::array::from_fn(DiagnosticsShard::for_index),
            last_stable_snapshot: Mutex::new(JobDiagnosticsSnapshot::default()),
            observation_journal: Arc::default(),
        }
    }
}

impl JobSchedulerDiagnosticsState {
    pub(super) fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    pub(super) fn task_diagnostic_source(&self) -> TaskDiagnosticSource {
        self.observation_enabled.store(true, Ordering::Release);
        TaskDiagnosticSource::new(Arc::clone(&self.observation_journal))
    }

    pub(super) fn task_identity(&self) -> Option<TaskDiagnosticIdentity> {
        self.observation_enabled.load(Ordering::Acquire).then(|| {
            TaskDiagnosticIdentity::new(
                self.observation_journal.source_id(),
                self.current_shard().allocate_task_sequence(),
            )
        })
    }

    pub(super) fn record_task_observation(
        &self,
        identity: Option<TaskDiagnosticIdentity>,
        kind: TaskDiagnosticKind,
        message: Arc<str>,
    ) {
        let Some(identity) = identity else {
            return;
        };
        self.observation_journal.record(identity, kind, message);
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

    pub(super) fn execution_started_at(&self, tracked: bool) -> Option<Instant> {
        tracked.then(Instant::now)
    }

    pub(super) fn record_active_terminal(
        &self,
        panicked: bool,
        execution_started_at: Option<Instant>,
    ) {
        let Some(execution_started_at) = execution_started_at else {
            return;
        };
        let shard = self.current_shard();
        let _update = shard.begin_update();
        add_duration_ns(&shard.execution_ns, execution_started_at.elapsed());
        shard.execution_samples.fetch_add(1, Ordering::Relaxed);
        if panicked {
            shard.panicked.fetch_add(1, Ordering::Relaxed);
        } else {
            shard.succeeded.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) fn record_active_cancelled(&self, execution_started_at: Option<Instant>) {
        let Some(execution_started_at) = execution_started_at else {
            return;
        };
        let shard = self.current_shard();
        let _update = shard.begin_update();
        add_duration_ns(&shard.execution_ns, execution_started_at.elapsed());
        shard.execution_samples.fetch_add(1, Ordering::Relaxed);
        shard.cancelled.fetch_add(1, Ordering::Relaxed);
        shard.cancelled_after_start.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_cancelled(&self, tracked: bool) {
        if !tracked {
            return;
        }
        let shard = self.current_shard();
        let _update = shard.begin_update();
        shard.cancelled.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_panicked(&self, tracked: bool) {
        if !tracked {
            return;
        }
        let shard = self.current_shard();
        let _update = shard.begin_update();
        shard.panicked.fetch_add(1, Ordering::Relaxed);
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
        self.try_stable_snapshot_with_attempt_hook(|_| {})
    }

    fn try_stable_snapshot_with_attempt_hook(
        &self,
        mut before_attempt: impl FnMut(usize),
    ) -> Option<JobDiagnosticsSnapshot> {
        for attempt in 0..MAX_AGGREGATE_SNAPSHOT_ATTEMPTS {
            before_attempt(attempt);
            if let Some(snapshot) = self.try_stable_snapshot_attempt() {
                return Some(snapshot);
            }

            std::hint::spin_loop();
        }

        None
    }

    fn try_stable_snapshot_attempt(&self) -> Option<JobDiagnosticsSnapshot> {
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
        self.shards
            .iter()
            .zip(epochs)
            .all(|(shard, epoch)| {
                shard.updates_in_flight.load(Ordering::Acquire) == 0
                    && shard.update_epoch.load(Ordering::Acquire) == epoch
            })
            .then_some(total)
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
    cancelled_after_start: AtomicU64,
    queue_wait_ns: AtomicU64,
    execution_ns: AtomicU64,
    execution_samples: AtomicU64,
    dependency_wait_ns: AtomicU64,
    explicit_wait_ns: AtomicU64,
    next_task_sequence: AtomicU64,
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
            cancelled_after_start: AtomicU64::new(0),
            queue_wait_ns: AtomicU64::new(0),
            execution_ns: AtomicU64::new(0),
            execution_samples: AtomicU64::new(0),
            dependency_wait_ns: AtomicU64::new(0),
            explicit_wait_ns: AtomicU64::new(0),
            next_task_sequence: AtomicU64::new(1),
        }
    }
}

impl DiagnosticsShard {
    fn for_index(index: usize) -> Self {
        Self {
            next_task_sequence: AtomicU64::new(index as u64 + 1),
            ..Self::default()
        }
    }

    fn allocate_task_sequence(&self) -> u64 {
        self.next_task_sequence
            .fetch_add(DIAGNOSTIC_SHARD_COUNT as u64, Ordering::Relaxed)
    }

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
                cancelled_after_start: self.cancelled_after_start.load(Ordering::Relaxed),
            },
            queue_wait_ns: self.queue_wait_ns.load(Ordering::Relaxed),
            execution_ns: self.execution_ns.load(Ordering::Relaxed),
            execution_samples: self.execution_samples.load(Ordering::Relaxed),
            dependency_wait_ns: self.dependency_wait_ns.load(Ordering::Relaxed),
            explicit_wait_ns: self.explicit_wait_ns.load(Ordering::Relaxed),
        }
    }
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

fn add_duration_ns(target: &AtomicU64, elapsed: Duration) {
    let elapsed_ns = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    target.fetch_add(elapsed_ns, Ordering::Relaxed);
}

pub(super) fn duration_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

#[cfg(test)]
#[path = "diagnostics/tests.rs"]
mod tests;
