use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime::core::runtime::tasks::TaskState;

use super::construction::CompletionBarrierGuard;
use super::{EditorJobSystem, ProgressObserverEvent};
use crate::core::jobs::event_sink::JobEventSink;
use crate::core::jobs::{
    CancellationToken, EditorJobAdmissionSnapshot, EditorJobProgressSource,
    EditorJobShutdownFailure, EditorJobShutdownProgress, EditorJobSpec, JobCategory, JobContext,
    JobEventKind, JobEventPumpBudget, JobId, UnfinishedEditorJob,
};

impl EditorJobSystem {
    pub fn pump_events(&self) -> usize {
        self.pump_events_with_budget(super::super::DEFAULT_JOB_EVENT_PUMP_BUDGET)
    }

    pub fn pump_events_with_budget(&self, budget: JobEventPumpBudget) -> usize {
        self.inner.lock_state().reap_execution_owners();
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
    ///
    /// The capability is crate-private so external domains cannot bypass job
    /// admission, progress, cancellation, and shutdown ownership. Internal
    /// process-capture adapters use it only while already executing a job.
    pub(crate) fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
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
        let execution = Arc::clone(&pending.execution);
        let cancel_task = pending.cancel_task;
        drop(state);
        let _completion = CompletionBarrierGuard::new(Arc::clone(&self.inner));
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
        execution.complete_without_task();
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
        // An empty progress map is not yet quiescent when terminal observer
        // delivery or promotion is still in flight.
        loop {
            state.reap_execution_owners();
            if !self.inner.progress.has_active()
                && !self.inner.completion_barrier_active()
                && !state.owned_tasks_pending()
                && state
                    .shutdown_cleanup()
                    .map_or(true, |cleanup| cleanup.is_complete())
            {
                break;
            }
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            // The completion barrier can notify before the cleanup handle
            // publishes terminal state. Recheck that handle within the deadline.
            let wait = deadline
                .saturating_duration_since(now)
                .min(Duration::from_millis(10));
            let (next_state, _) = self
                .inner
                .state_changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
        }
        self.inner.progress.unfinished_jobs()
    }

    /// Closes editor admission and moves pending cancellation onto one owned runtime job.
    ///
    /// The state mutex is the start gate: the worker cannot invoke cancellation
    /// until its real handle is installed. Submission failure remains observable.
    pub fn request_shutdown(&self) {
        let mut state = self.inner.lock_state();
        if state.shutdown_requested() {
            return;
        }
        let pending = state.begin_shutdown();
        self.inner.progress.cancel_all();
        if pending.is_empty() {
            return;
        }

        let executions = pending
            .iter()
            .map(|pending| Arc::clone(&pending.execution))
            .collect::<Vec<_>>();
        let completion = CompletionBarrierGuard::new(Arc::clone(&self.inner));
        let inner = Arc::clone(&self.inner);
        let cleanup = catch_unwind(AssertUnwindSafe(|| {
            self.inner.scheduler.schedule_after(&[], move || {
                let _completion = completion;
                let may_run = {
                    let state = inner.lock_state();
                    state.shutdown_cleanup_may_run()
                };
                if !may_run {
                    return;
                }
                inner.cancel_pending(pending);
            })
        }));
        match cleanup {
            Ok(cleanup) => {
                for execution in &executions {
                    execution.bind_cleanup(&cleanup);
                }
                state.install_shutdown_cleanup(cleanup);
            }
            Err(payload) => {
                let message = if let Some(message) = payload.downcast_ref::<&str>() {
                    (*message).to_owned()
                } else if let Some(message) = payload.downcast_ref::<String>() {
                    message.clone()
                } else {
                    "editor shutdown cleanup submission panicked".to_owned()
                };
                for execution in &executions {
                    execution.fail_submission(&message);
                }
                state.record_shutdown_cleanup_failure(
                    EditorJobShutdownFailure::CleanupSubmissionPanicked { message },
                );
            }
        }
    }

    /// Polls ownership and terminal side effects without waiting or invoking user code.
    pub fn shutdown_progress(&self) -> EditorJobShutdownProgress {
        let mut state = self.inner.lock_state();
        state.reap_execution_owners();
        let cleanup_terminal = state
            .shutdown_cleanup()
            .map(|cleanup| cleanup.terminal_state());
        let cleanup_pending = matches!(cleanup_terminal, Some(None));
        let cleanup_failure = state.shutdown_cleanup_failure().cloned().or_else(|| {
            match cleanup_terminal.flatten() {
                Some(TaskState::Failed) => Some(EditorJobShutdownFailure::CleanupTaskFailed),
                Some(TaskState::Cancelled) => Some(EditorJobShutdownFailure::CleanupCancelled),
                _ => None,
            }
        });
        EditorJobShutdownProgress::new(
            state.shutdown_requested(),
            self.inner.progress.unfinished_jobs(),
            self.inner.completion_barrier_active(),
            state.owned_tasks_pending(),
            cleanup_pending,
            cleanup_failure,
        )
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
#[path = "lifecycle/tests/owned_cancel_metadata.rs"]
mod owned_cancel_metadata_tests;

#[cfg(test)]
mod editor09_owned_shutdown_tests {
    use super::super::pending::PendingJob;
    use super::super::pending_task::PendingTask;
    use super::*;
    use crate::core::jobs::{
        test_job_scheduler, test_job_system, EditorJobLimits, EditorJobProgressObserver,
        EditorJobShutdownState,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;

    fn enqueue(
        jobs: &EditorJobSystem,
        task: Box<dyn PendingTask>,
        cancel: Box<dyn FnOnce(JobContext) + Send + 'static>,
    ) -> JobId {
        let mut state = jobs.inner.lock_state();
        let id = state.allocate_id();
        let spec = EditorJobSpec::new("owned shutdown fixture", JobCategory::Misc);
        state.register(id);
        jobs.inner.progress.register(id, &spec);
        state.enqueue_pending(PendingJob::new(id, spec, task, cancel, Instant::now()));
        id
    }

    fn wait_until(mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready() {
            assert!(
                Instant::now() < deadline,
                "owned shutdown did not become terminal"
            );
            std::thread::yield_now();
        }
    }

    #[test]
    fn cleanup_installs_real_handle_before_callback_and_polls_until_terminal() {
        let jobs = test_job_system();
        let callback_jobs = jobs.clone();
        let caller = std::thread::current().id();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_calls = Arc::clone(&calls);
        enqueue(
            &jobs,
            Box::new(|_: JobContext| {}),
            Box::new(move |_| {
                callback_calls.fetch_add(1, Ordering::AcqRel);
                let installed = callback_jobs
                    .inner
                    .lock_state()
                    .shutdown_cleanup()
                    .is_some();
                started_tx
                    .send((std::thread::current().id(), installed))
                    .unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            }),
        );

        jobs.request_shutdown();
        let (callback_thread, installed) = started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_ne!(callback_thread, caller);
        assert!(installed);
        let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
        assert!(!cleanup.is_complete());
        assert_eq!(
            jobs.shutdown_progress().state(),
            EditorJobShutdownState::Pending
        );
        jobs.request_shutdown();
        release_tx.send(()).unwrap();
        wait_until(|| jobs.shutdown_progress().state() == EditorJobShutdownState::Quiescent);
        assert_eq!(cleanup.terminal_state(), Some(TaskState::Completed));
        assert_eq!(calls.load(Ordering::Acquire), 1);
    }

    #[test]
    fn empty_progress_stays_pending_while_real_completion_barrier_is_owned() {
        let jobs = test_job_system();
        let barrier = CompletionBarrierGuard::new(Arc::clone(&jobs.inner));
        jobs.request_shutdown();
        let pending = jobs.shutdown_progress();
        assert!(pending.unfinished_jobs().is_empty());
        assert!(pending.completion_callbacks_pending());
        assert_eq!(pending.state(), EditorJobShutdownState::Pending);
        drop(barrier);
        assert_eq!(
            jobs.shutdown_progress().state(),
            EditorJobShutdownState::Quiescent
        );
    }

    #[test]
    fn cleanup_runtime_failure_is_sticky_and_never_quiescent() {
        let jobs = test_job_system();
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_calls = Arc::clone(&calls);
        enqueue(
            &jobs,
            Box::new(|_: JobContext| {}),
            Box::new(move |_| {
                callback_calls.fetch_add(1, Ordering::AcqRel);
                panic!("real cleanup callback failure");
            }),
        );
        jobs.request_shutdown();
        let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
        wait_until(|| cleanup.is_complete());
        assert_eq!(cleanup.terminal_state(), Some(TaskState::Failed));
        let failed = jobs.shutdown_progress();
        assert_eq!(failed.state(), EditorJobShutdownState::Failed);
        assert!(matches!(
            failed.cleanup_failure_kind(),
            Some(EditorJobShutdownFailure::CleanupTaskFailed)
        ));
        jobs.request_shutdown();
        assert_eq!(calls.load(Ordering::Acquire), 1);
        assert_eq!(
            jobs.shutdown_progress().state(),
            EditorJobShutdownState::Failed
        );
    }

    #[derive(Default)]
    struct FinishedObserver(AtomicUsize);

    impl EditorJobProgressObserver for FinishedObserver {
        fn job_admitted(&self, _: JobId, _: &EditorJobProgressSource) {}
        fn job_finished(&self, _: JobId, _: &EditorJobProgressSource) {
            self.0.fetch_add(1, Ordering::AcqRel);
        }
        fn jobs_resynchronized(&self, _: &EditorJobProgressSource) {}
    }

    #[test]
    fn prior_real_task_failure_does_not_skip_remaining_cancel_or_finished_observer() {
        let observer = Arc::new(FinishedObserver::default());
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            test_job_scheduler(),
            Default::default(),
            EditorJobLimits::default(),
            observer.clone(),
        );
        let failed_id = enqueue(
            &jobs,
            Box::new(|_: JobContext| {
                panic!("raw runtime task failure outside EditorJob result wrapper");
            }),
            Box::new(|_| {}),
        );
        jobs.inner.promote();
        wait_until(|| {
            matches!(
                jobs.shutdown_progress().cleanup_failure_kind(),
                Some(EditorJobShutdownFailure::JobTaskFailed { job, .. }) if *job == failed_id
            )
        });
        let finished_before = observer.0.load(Ordering::Acquire);
        let calls = Arc::new(AtomicUsize::new(0));
        let cancel_calls = Arc::clone(&calls);
        let pending_id = enqueue(
            &jobs,
            Box::new(|_: JobContext| {}),
            Box::new(move |_| {
                cancel_calls.fetch_add(1, Ordering::AcqRel);
            }),
        );
        jobs.request_shutdown();
        let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
        wait_until(|| cleanup.is_complete());
        assert_eq!(cleanup.terminal_state(), Some(TaskState::Completed));
        assert_eq!(calls.load(Ordering::Acquire), 1);
        assert!(jobs.is_terminal_record(pending_id));
        assert_eq!(observer.0.load(Ordering::Acquire), finished_before + 1);
        assert_eq!(
            jobs.shutdown_progress().state(),
            EditorJobShutdownState::Failed
        );
        jobs.request_shutdown();
        assert_eq!(calls.load(Ordering::Acquire), 1);
    }
}
