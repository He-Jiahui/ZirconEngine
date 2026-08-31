use std::sync::Arc;
use std::time::Instant;

use super::{EditorJobSystem, ProgressObserverEvent};
use crate::core::jobs::event_sink::JobEventSink;
use crate::core::jobs::{
    CancellationToken, EditorJobAdmissionSnapshot, EditorJobProgressSource, EditorJobSpec,
    JobCategory, JobContext, JobEventKind, JobEventPumpBudget, JobId, UnfinishedEditorJob,
};

impl EditorJobSystem {
    pub fn pump_events(&self) -> usize {
        self.pump_events_with_budget(super::super::DEFAULT_JOB_EVENT_PUMP_BUDGET)
    }

    pub fn pump_events_with_budget(&self, budget: JobEventPumpBudget) -> usize {
        let pumped = self.inner.event_pump.pump(budget);
        self.inner.deliver_progress_observer_events();
        pumped
    }

    #[cfg(test)]
    pub(crate) fn pump_events_with_elapsed(
        &self,
        budget: JobEventPumpBudget,
        elapsed: impl FnMut() -> std::time::Duration,
    ) -> usize {
        self.inner.event_pump.pump_with_elapsed(budget, elapsed)
    }

    pub fn progress(&self) -> EditorJobProgressSource {
        self.inner.progress.clone()
    }

    pub fn admission_snapshot(&self) -> EditorJobAdmissionSnapshot {
        self.inner.lock_state().admission_snapshot(Instant::now())
    }

    #[cfg(test)]
    pub(crate) fn category_admission_snapshot(
        &self,
        category: crate::core::jobs::JobCategory,
    ) -> EditorJobAdmissionSnapshot {
        self.inner
            .lock_state()
            .category_admission_snapshot(category, Instant::now())
    }

    /// Runs two borrowing tasks through the shared runtime scheduler without creating editor threads.
    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.inner.scheduler.join(task_a, task_b)
    }

    /// Cancels a pending job synchronously or requests cooperative cancellation from active work.
    pub fn cancel(&self, id: JobId) -> bool {
        let mut state = self.inner.lock_state();
        let Some(pending) = state.remove_pending(id) else {
            return self.inner.progress.request_cancel(id);
        };
        let PendingCancelMetadata {
            label,
            category,
            cancel,
        } = into_pending_cancel_metadata(pending.spec);
        cancel.cancel();
        let cancel_task = pending.cancel_task;
        drop(state);
        let events = JobEventSink::new(
            id,
            label,
            category,
            self.inner.event_queue.clone(),
            self.inner.progress.clone(),
        );
        cancel_task(JobContext::new(cancel, events));
        let mut state = self.inner.lock_state();
        state.mark_cancelled(id);
        self.inner.progress.complete(id);
        self.inner
            .enqueue_progress_observer_event(ProgressObserverEvent::Finished(id));
        drop(state);
        self.inner.deliver_progress_observer_events();
        self.inner.state_changed.notify_all();
        self.inner.promote();
        true
    }

    pub fn shutdown(&self, deadline: Instant) -> Vec<UnfinishedEditorJob> {
        let pending = {
            let mut state = self.inner.lock_state();
            let pending = state.begin_shutdown();
            self.inner.progress.cancel_all();
            pending
        };
        self.inner.cancel_pending(pending);

        let mut state = self.inner.lock_state();
        while self.inner.progress.has_active() {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let wait = deadline.saturating_duration_since(now);
            let (next_state, timeout) = self
                .inner
                .state_changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if timeout.timed_out() {
                break;
            }
        }
        self.inner.progress.unfinished_jobs()
    }

    #[cfg(test)]
    pub(crate) fn retained_record_count(&self) -> usize {
        self.inner.lock_state().retained_record_count()
    }

    #[cfg(test)]
    pub(crate) fn is_terminal_record(&self, id: JobId) -> bool {
        self.inner.lock_state().is_terminal_record(id)
    }

    #[cfg(test)]
    pub(crate) fn scheduled_record_count(&self) -> usize {
        self.inner.lock_state().scheduled_record_count()
    }

    #[cfg(test)]
    pub(crate) fn pending_job_count(&self) -> usize {
        self.inner.lock_state().pending_len()
    }

    #[cfg(test)]
    pub(crate) fn running_job_count(&self) -> usize {
        self.inner.lock_state().running_job_count()
    }

    #[cfg(test)]
    pub(crate) fn mutex_group_tail_count(&self) -> usize {
        self.inner.lock_state().mutex_group_tail_count()
    }

    #[cfg(test)]
    pub(crate) fn admission_probe_count(&self) -> usize {
        self.inner.lock_state().admission_probe_count()
    }

    #[cfg(test)]
    pub(crate) const fn terminal_record_retention_limit(&self) -> usize {
        super::state::TERMINAL_RECORD_RETENTION_LIMIT
    }
}

struct PendingCancelMetadata {
    label: Arc<str>,
    category: JobCategory,
    cancel: CancellationToken,
}

fn into_pending_cancel_metadata(spec: EditorJobSpec) -> PendingCancelMetadata {
    let EditorJobSpec {
        label,
        category,
        cancel,
        ..
    } = spec;
    PendingCancelMetadata {
        label,
        category,
        cancel,
    }
}

#[cfg(test)]
#[path = "lifecycle/owned_cancel_metadata_tests.rs"]
mod owned_cancel_metadata_tests;
