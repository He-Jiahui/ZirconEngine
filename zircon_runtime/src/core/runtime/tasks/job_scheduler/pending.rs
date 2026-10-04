use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use super::JobExecutionOutcome;
use crate::core::runtime::tasks::{
    JobHandle, JobSchedulerDiagnosticsState, TaskDiagnosticIdentity, TaskDiagnosticKind,
    TaskPoolSubmission,
};

pub(super) type PostTerminalHook = Box<dyn FnOnce(JobExecutionOutcome) + Send + 'static>;

pub(super) type ScheduledJob = Box<dyn FnOnce() -> JobExecutionOutcome + Send + 'static>;
pub(super) type PrelaunchTerminalHook =
    Box<dyn FnOnce(TaskDiagnosticKind, Arc<str>) + Send + 'static>;

pub(super) struct PendingScheduledWork {
    pub(super) task: ScheduledJob,
    pub(super) submission: TaskPoolSubmission,
    pub(super) prelaunch_terminal: Option<PrelaunchTerminalHook>,
    pub(super) post_terminal: Option<PostTerminalHook>,
}

pub(super) struct PendingScheduledJob {
    pub(super) handle: JobHandle,
    pub(super) diagnostics: Arc<JobSchedulerDiagnosticsState>,
    pub(super) identity: Option<TaskDiagnosticIdentity>,
    pub(super) created_at: Option<Instant>,
    pub(super) diagnostics_tracked: bool,
    pub(super) dependency_count: usize,
    pub(super) work: Mutex<Option<PendingScheduledWork>>,
}

impl PendingScheduledJob {
    pub(super) fn has_pending_work(&self) -> bool {
        self.lock_work().is_some()
    }

    // 从 work 槽取出闭包是依赖完成到实际提交的单次线性化点；槽已空表示终态路径已消费该工作。
    pub(super) fn try_launch(&self) {
        let Some(work) = self.lock_work().take() else {
            return;
        };
        if self.dependency_count > 0 {
            self.diagnostics.record_dependency_wait(self.created_at);
        }
        let handle = self.handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        let identity = self.identity;
        let enqueued_at = diagnostics.record_enqueued(self.diagnostics_tracked);
        work.submission.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            complete_scheduled_task(
                handle,
                diagnostics,
                identity,
                execution_started_at,
                work.task,
                work.post_terminal,
            );
        });
    }

    pub(super) fn record_terminal_without_launch(
        &self,
        kind: TaskDiagnosticKind,
        message: Arc<str>,
    ) -> bool {
        let pending_work = self.lock_work().take();
        let Some(mut work) = pending_work else {
            return false;
        };
        if self.dependency_count > 0 {
            self.diagnostics.record_dependency_wait(self.created_at);
        }
        match kind {
            TaskDiagnosticKind::Cancelled => {
                self.diagnostics.record_cancelled(self.diagnostics_tracked);
            }
            TaskDiagnosticKind::Panicked => {
                self.diagnostics.record_panicked(self.diagnostics_tracked);
            }
        }
        self.diagnostics
            .record_task_observation(self.identity, kind, Arc::clone(&message));
        match kind {
            TaskDiagnosticKind::Cancelled => self.handle.mark_cancelled(),
            TaskDiagnosticKind::Panicked => self.handle.mark_panicked(Arc::clone(&message)),
        }
        if let Some(prelaunch_terminal) = work.prelaunch_terminal.take() {
            prelaunch_terminal(kind, Arc::clone(&message));
        }
        drop(work);
        true
    }

    fn lock_work(&self) -> MutexGuard<'_, Option<PendingScheduledWork>> {
        self.work
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn complete_scheduled_task(
    handle: JobHandle,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    identity: Option<TaskDiagnosticIdentity>,
    execution_started_at: Option<Instant>,
    task: impl FnOnce() -> JobExecutionOutcome,
    post_terminal: Option<PostTerminalHook>,
) {
    handle.mark_running();
    let result = catch_unwind(AssertUnwindSafe(task));
    match result {
        Ok(JobExecutionOutcome::Completed) => {
            diagnostics.record_active_terminal(false, execution_started_at);
            handle.mark_complete();
            if let Some(post_terminal) = post_terminal {
                post_terminal(JobExecutionOutcome::Completed);
            }
        }
        Ok(JobExecutionOutcome::Cancelled) => {
            diagnostics.record_active_cancelled(execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Cancelled,
                Arc::from("task cancellation acknowledged"),
            );
            handle.mark_cancelled();
            if let Some(post_terminal) = post_terminal {
                post_terminal(JobExecutionOutcome::Cancelled);
            }
        }
        Ok(JobExecutionOutcome::Panicked(message)) => {
            diagnostics.record_active_terminal(true, execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Panicked,
                Arc::clone(&message),
            );
            let outcome = JobExecutionOutcome::Panicked(Arc::clone(&message));
            handle.mark_panicked(message);
            if let Some(post_terminal) = post_terminal {
                post_terminal(outcome);
            }
        }
        Err(payload) => {
            let message = panic_payload_message(payload);
            diagnostics.record_active_terminal(true, execution_started_at);
            diagnostics.record_task_observation(
                identity,
                TaskDiagnosticKind::Panicked,
                Arc::clone(&message),
            );
            let outcome = JobExecutionOutcome::Panicked(Arc::clone(&message));
            handle.mark_panicked(message);
            if let Some(post_terminal) = post_terminal {
                post_terminal(outcome);
            }
        }
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> Arc<str> {
    if let Some(message) = payload.downcast_ref::<&str>() {
        Arc::from(*message)
    } else if let Some(message) = payload.downcast_ref::<String>() {
        Arc::from(message.as_str())
    } else {
        Arc::from("non-string panic payload")
    }
}
