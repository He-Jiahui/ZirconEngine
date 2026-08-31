//! Runtime scheduler facade for compute work submitted through the core task pools.

mod pending;

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::core::diagnostics::DiagnosticStore;

use super::callback_dispatcher::TaskCallbackDispatcher;
use super::{
    JobHandle, JobSchedulerDiagnosticsState, JobSchedulerReport, TaskDiagnosticIdentity,
    TaskDiagnosticKind, TaskDiagnosticSource, TaskPool, TaskPoolKind, TaskPoolSubmission,
};
use pending::{PendingScheduledJob, PendingScheduledWork, PrelaunchTerminalHook};

#[derive(Clone)]
pub struct JobScheduler {
    pool: TaskPool,
    callback_dispatcher: TaskCallbackDispatcher,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
}

impl fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobScheduler")
            .field("parallelism", &self.parallelism())
            .finish()
    }
}

impl JobScheduler {
    /// Creates a scheduling facade over an existing task-pool owner.
    ///
    /// This does not allocate another worker budget. The supplied pool remains governed by its
    /// owning [`EngineTaskGraph`](super::EngineTaskGraph) or explicit [`TaskPool`] owner.
    pub fn from_pool(pool: TaskPool) -> Self {
        Self::from_pool_with_callback_dispatcher(pool.clone(), TaskCallbackDispatcher::new(pool))
    }

    fn from_pool_with_callback_dispatcher(
        pool: TaskPool,
        callback_dispatcher: TaskCallbackDispatcher,
    ) -> Self {
        Self {
            callback_dispatcher,
            pool,
            diagnostics: Arc::default(),
        }
    }

    /// Enables bounded lifecycle diagnostics before work is submitted to this scheduler.
    pub fn with_diagnostics(self) -> Self {
        self.diagnostics.enable();
        self
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        let submission = self.submission_or_panic();
        let diagnostics = Arc::clone(&self.diagnostics);
        let enqueued_at = diagnostics.record_scheduled_and_enqueued();
        let identity = diagnostics.task_identity();
        submission.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            run_detached_task(diagnostics, identity, execution_started_at, task);
        });
    }

    pub fn schedule(&self, task: impl FnOnce() + Send + 'static) -> JobHandle {
        self.schedule_with_outcome(move || {
            task();
            JobExecutionOutcome::Completed
        })
    }

    pub(super) fn schedule_with_outcome(
        &self,
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
    ) -> JobHandle {
        let submission = self.submission_or_panic();
        self.schedule_with_submission(submission, task)
    }

    pub(super) fn schedule_with_submission(
        &self,
        submission: TaskPoolSubmission,
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
    ) -> JobHandle {
        let handle = JobHandle::pending_with_scheduler_diagnostics(
            0,
            Arc::clone(&self.diagnostics),
            self.callback_dispatcher.clone(),
        );
        let handle_for_task = handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        let enqueued_at = diagnostics.record_scheduled_and_enqueued();
        let identity = diagnostics.task_identity();
        submission.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            complete_scheduled_task(
                handle_for_task,
                diagnostics,
                identity,
                execution_started_at,
                task,
            );
        });
        handle
    }

    pub fn schedule_after(
        &self,
        dependencies: &[JobHandle],
        task: impl FnOnce() + Send + 'static,
    ) -> JobHandle {
        self.schedule_after_with_outcome(dependencies, move || {
            task();
            JobExecutionOutcome::Completed
        })
    }

    pub(super) fn schedule_after_with_outcome(
        &self,
        dependencies: &[JobHandle],
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
    ) -> JobHandle {
        let submission = self.submission_or_panic();
        self.schedule_after_with_submission(dependencies, submission, task)
    }

    pub(super) fn schedule_after_with_submission(
        &self,
        dependencies: &[JobHandle],
        submission: TaskPoolSubmission,
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
    ) -> JobHandle {
        self.schedule_after_with_submission_inner(dependencies, submission, task, None)
    }

    pub(super) fn schedule_after_with_submission_and_prelaunch_terminal(
        &self,
        dependencies: &[JobHandle],
        submission: TaskPoolSubmission,
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
        prelaunch_terminal: impl FnOnce(TaskDiagnosticKind, Arc<str>) + Send + 'static,
    ) -> JobHandle {
        self.schedule_after_with_submission_inner(
            dependencies,
            submission,
            task,
            Some(Box::new(prelaunch_terminal)),
        )
    }

    fn schedule_after_with_submission_inner(
        &self,
        dependencies: &[JobHandle],
        submission: TaskPoolSubmission,
        task: impl FnOnce() -> JobExecutionOutcome + Send + 'static,
        prelaunch_terminal: Option<PrelaunchTerminalHook>,
    ) -> JobHandle {
        if dependencies.is_empty() {
            return self.schedule_with_submission(submission, task);
        }

        let diagnostics_tracked = self.diagnostics.record_scheduled();
        let identity = self.diagnostics.task_identity();
        let handle = JobHandle::pending_with_scheduler_diagnostics(
            dependencies.len(),
            Arc::clone(&self.diagnostics),
            self.callback_dispatcher.clone(),
        );
        let pending = Arc::new(PendingScheduledJob {
            handle: handle.clone(),
            diagnostics: Arc::clone(&self.diagnostics),
            identity,
            created_at: diagnostics_tracked.then(Instant::now),
            diagnostics_tracked,
            dependency_count: dependencies.len(),
            work: Mutex::new(Some(PendingScheduledWork {
                task: Box::new(task),
                submission,
                prelaunch_terminal,
            })),
        });

        for dependency in dependencies {
            let dependency_for_callback = dependency.clone();
            let handle_for_callback = handle.clone();
            let pending_for_callback = Arc::clone(&pending);
            let callback = Box::new(move || {
                if let Some(panic_message) = dependency_for_callback.panic_message() {
                    pending_for_callback.record_terminal_without_launch(
                        TaskDiagnosticKind::Panicked,
                        Arc::clone(&panic_message),
                    );
                    return;
                }
                if dependency_for_callback.is_cancelled() {
                    pending_for_callback.record_terminal_without_launch(
                        TaskDiagnosticKind::Cancelled,
                        Arc::from("dependency cancelled before task launch"),
                    );
                    return;
                }
                if handle_for_callback.dependency_completed() {
                    pending_for_callback.try_launch();
                }
            });
            if !dependency.add_dependent(callback) {
                if let Some(panic_message) = dependency.panic_message() {
                    pending.record_terminal_without_launch(
                        TaskDiagnosticKind::Panicked,
                        Arc::clone(&panic_message),
                    );
                } else if dependency.is_cancelled() {
                    pending.record_terminal_without_launch(
                        TaskDiagnosticKind::Cancelled,
                        Arc::from("dependency cancelled before task launch"),
                    );
                } else if handle.dependency_completed() {
                    pending.try_launch();
                }
            }
        }

        handle
    }

    pub fn wait_all(&self, handles: &[JobHandle]) {
        JobHandle::combine_with_scheduler_diagnostics(handles, Arc::clone(&self.diagnostics))
            .wait();
    }

    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.pool.install(task)
    }

    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.pool.join(task_a, task_b)
    }

    pub fn parallelism(&self) -> usize {
        self.pool.parallelism()
    }

    pub(super) fn pool_kind(&self) -> TaskPoolKind {
        self.pool.kind()
    }

    pub(super) fn shares_execution_owner_with(&self, pool: &TaskPool) -> bool {
        self.pool.shares_execution_owner_with(pool)
    }

    fn submission_or_panic(&self) -> TaskPoolSubmission {
        self.pool
            .try_acquire_submission()
            .unwrap_or_else(|| panic!("{:?} task scheduler is closing admission", self.pool.kind()))
    }

    pub fn diagnostic_report(&self) -> JobSchedulerReport {
        self.diagnostics.report()
    }

    /// Enables task diagnostics and returns a bounded runtime-neutral observation source.
    pub fn task_diagnostic_source(&self) -> TaskDiagnosticSource {
        self.diagnostics.task_diagnostic_source()
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.diagnostics.record_diagnostics(store, frame_index);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum JobExecutionOutcome {
    Completed,
    Cancelled,
    Panicked(Arc<str>),
}

struct DetachedTaskCompletion {
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    identity: Option<TaskDiagnosticIdentity>,
    execution_started_at: Option<Instant>,
}

impl DetachedTaskCompletion {
    fn new(
        diagnostics: Arc<JobSchedulerDiagnosticsState>,
        identity: Option<TaskDiagnosticIdentity>,
        execution_started_at: Option<Instant>,
    ) -> Self {
        Self {
            diagnostics,
            identity,
            execution_started_at,
        }
    }
}

impl Drop for DetachedTaskCompletion {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.diagnostics.record_task_observation(
                self.identity,
                TaskDiagnosticKind::Panicked,
                Arc::from("detached task panicked"),
            );
        }
        self.diagnostics
            .record_active_terminal(std::thread::panicking(), self.execution_started_at);
    }
}

fn run_detached_task(
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    identity: Option<TaskDiagnosticIdentity>,
    execution_started_at: Option<Instant>,
    task: impl FnOnce(),
) {
    let _completion = DetachedTaskCompletion::new(diagnostics, identity, execution_started_at);
    task();
}

fn complete_scheduled_task(
    handle: JobHandle,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    identity: Option<TaskDiagnosticIdentity>,
    execution_started_at: Option<Instant>,
    task: impl FnOnce() -> JobExecutionOutcome,
) {
    handle.mark_running();
    let result = catch_unwind(AssertUnwindSafe(task));
    match result {
        Ok(JobExecutionOutcome::Completed) => {
            diagnostics.record_active_terminal(false, execution_started_at);
            handle.mark_complete();
        }
        Ok(JobExecutionOutcome::Cancelled) => {
            diagnostics.record_active_cancelled(execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Cancelled,
                Arc::from("task cancellation acknowledged"),
            );
            handle.mark_cancelled();
        }
        Ok(JobExecutionOutcome::Panicked(message)) => {
            diagnostics.record_active_terminal(true, execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Panicked,
                Arc::clone(&message),
            );
            handle.mark_panicked(message);
        }
        Err(payload) => {
            let message = panic_payload_message(payload);
            diagnostics.record_active_terminal(true, execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Panicked,
                Arc::clone(&message),
            );
            handle.mark_panicked(message);
        }
    }
}

#[cfg(test)]
#[path = "job_scheduler/tests.rs"]
mod tests;

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> Arc<str> {
    if let Some(message) = payload.downcast_ref::<&str>() {
        Arc::from(*message)
    } else if let Some(message) = payload.downcast_ref::<String>() {
        Arc::from(message.as_str())
    } else {
        Arc::from("non-string panic payload")
    }
}
