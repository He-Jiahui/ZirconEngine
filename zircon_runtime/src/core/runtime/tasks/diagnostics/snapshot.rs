use super::{duration_ms, JobSchedulerReport};

#[derive(Clone, Copy, Debug)]
pub(super) struct StableDiagnosticsSnapshot {
    pub(super) epoch: u64,
    pub(super) diagnostics: JobDiagnosticsSnapshot,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct JobDiagnosticsSnapshot {
    pub(super) lifecycle: JobLifecycleSnapshot,
    pub(super) queue_wait_ns: u64,
    pub(super) execution_ns: u64,
    pub(super) execution_samples: u64,
    pub(super) dependency_wait_ns: u64,
    pub(super) explicit_wait_ns: u64,
}

impl JobDiagnosticsSnapshot {
    pub(super) fn merge(&mut self, other: Self) {
        self.lifecycle.merge(other.lifecycle);
        self.queue_wait_ns = self.queue_wait_ns.saturating_add(other.queue_wait_ns);
        self.execution_ns = self.execution_ns.saturating_add(other.execution_ns);
        self.execution_samples = self
            .execution_samples
            .saturating_add(other.execution_samples);
        self.dependency_wait_ns = self
            .dependency_wait_ns
            .saturating_add(other.dependency_wait_ns);
        self.explicit_wait_ns = self.explicit_wait_ns.saturating_add(other.explicit_wait_ns);
    }

    pub(super) fn report(self) -> JobSchedulerReport {
        let lifecycle = self.lifecycle;
        JobSchedulerReport {
            scheduled: lifecycle.scheduled,
            completed: lifecycle.completed(),
            dependency_waiting: lifecycle.dependency_waiting(),
            queued: lifecycle.queued(),
            active: lifecycle.active(),
            queue_wait_samples: lifecycle.started,
            queue_wait_ms: duration_ms(self.queue_wait_ns),
            execution_samples: self.execution_samples,
            execution_ms: duration_ms(self.execution_ns),
            panicked: lifecycle.panicked,
            cancelled: lifecycle.cancelled,
            dependency_wait_ms: duration_ms(self.dependency_wait_ns),
            explicit_wait_ms: duration_ms(self.explicit_wait_ns),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct JobLifecycleSnapshot {
    pub(super) scheduled: u64,
    pub(super) enqueued: u64,
    pub(super) started: u64,
    pub(super) succeeded: u64,
    pub(super) panicked: u64,
    pub(super) cancelled: u64,
    pub(super) cancelled_after_start: u64,
}

impl JobLifecycleSnapshot {
    fn merge(&mut self, other: Self) {
        self.scheduled = self.scheduled.saturating_add(other.scheduled);
        self.enqueued = self.enqueued.saturating_add(other.enqueued);
        self.started = self.started.saturating_add(other.started);
        self.succeeded = self.succeeded.saturating_add(other.succeeded);
        self.panicked = self.panicked.saturating_add(other.panicked);
        self.cancelled = self.cancelled.saturating_add(other.cancelled);
        self.cancelled_after_start = self
            .cancelled_after_start
            .saturating_add(other.cancelled_after_start);
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
        self.started.saturating_sub(
            self.succeeded
                .saturating_add(self.panicked)
                .saturating_add(self.cancelled_after_start),
        )
    }
}
