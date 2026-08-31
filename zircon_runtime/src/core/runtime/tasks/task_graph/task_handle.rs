use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use super::super::{JobHandle, JobScheduler, TaskCancellationPolicy, TaskDescriptor, TaskStatus};
use super::lease::TaskGraphClientLease;
use super::scope::{panic_payload_message, TaskCancellationToken, TaskGraphScopeInner};
use super::scope_model::TaskGraphScopeCensus;

pub(super) struct TaskRecord {
    pub(super) descriptor: TaskDescriptor,
    state: Mutex<TaskRecordState>,
}

pub(super) struct TaskRecordState {
    pub(super) cancellation_requested: bool,
    pub(super) cancellation_acknowledged: bool,
}

/// Canonical handle for descriptor-led work admitted to the Runtime task owner.
pub struct TaskHandle {
    pub(super) record: Arc<TaskRecord>,
    pub(super) scope: Option<Arc<TaskGraphScopeInner>>,
    pub(super) completion: JobHandle,
    pub(super) handle_lease: Arc<TaskGraphClientLease>,
}

impl TaskHandle {
    pub(crate) fn schedule_detached(
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Self {
        let record = Arc::new(TaskRecord::new(descriptor));
        let record_for_task = Arc::clone(&record);
        let completion =
            scheduler.schedule_with_outcome(move || record_for_task.run_detached(task));
        Self {
            record,
            scope: None,
            completion,
            handle_lease: TaskGraphClientLease::new(),
        }
    }

    pub(crate) fn completed(descriptor: TaskDescriptor) -> Self {
        let record = Arc::new(TaskRecord::new(descriptor));
        Self {
            record,
            scope: None,
            completion: JobHandle::completed(),
            handle_lease: TaskGraphClientLease::new(),
        }
    }

    pub fn descriptor(&self) -> &TaskDescriptor {
        &self.record.descriptor
    }

    pub fn status(&self) -> TaskStatus {
        self.completion.task_status(self.record.descriptor.id)
    }

    pub fn is_complete(&self) -> bool {
        self.completion.is_complete()
    }

    pub fn is_cancelled(&self) -> bool {
        self.completion.is_cancelled()
    }

    pub fn wait(&self) {
        self.completion.wait();
    }

    pub fn on_terminal(&self, observer: impl FnOnce() + Send + 'static) {
        self.completion.on_terminal(observer);
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.record.lock_state().cancellation_requested
    }

    /// Requests cooperative cancellation. Running work observes the request
    /// through its token; queued work converts to `Cancelled` when its worker
    /// closure is reached, preserving queue-drain accounting.
    pub fn request_cancellation(&self) {
        if !self.completion.is_complete() {
            self.record.request_cancellation();
        }
    }

    pub fn scope_census(&self) -> TaskGraphScopeCensus {
        self.scope
            .as_ref()
            .expect("public task handles are scope-owned")
            .census()
    }
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        self.handle_lease.retain();
        Self {
            record: Arc::clone(&self.record),
            scope: self.scope.clone(),
            completion: self.completion.clone(),
            handle_lease: Arc::clone(&self.handle_lease),
        }
    }
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskHandle")
            .field("descriptor", self.descriptor())
            .field("status", &self.status())
            .field("is_complete", &self.is_complete())
            .finish()
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        if self.handle_lease.release()
            && self.record.descriptor.cancellation_policy == TaskCancellationPolicy::CancelOnDrop
        {
            self.request_cancellation();
        }
    }
}

impl TaskRecord {
    pub(super) fn new(descriptor: TaskDescriptor) -> Self {
        Self {
            state: Mutex::new(TaskRecordState {
                cancellation_requested: false,
                cancellation_acknowledged: false,
            }),
            descriptor,
        }
    }

    fn run_detached(
        self: Arc<Self>,
        task: impl FnOnce(TaskCancellationToken),
    ) -> super::super::job_scheduler::JobExecutionOutcome {
        {
            let state = self.lock_state();
            if state.cancellation_requested {
                return super::super::job_scheduler::JobExecutionOutcome::Cancelled;
            }
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            task(TaskCancellationToken {
                record: Arc::clone(&self),
            });
        }));
        let mut state = self.lock_state();
        match result {
            Ok(()) if state.cancellation_acknowledged => {
                super::super::job_scheduler::JobExecutionOutcome::Cancelled
            }
            Ok(()) => super::super::job_scheduler::JobExecutionOutcome::Completed,
            Err(payload) => super::super::job_scheduler::JobExecutionOutcome::Panicked(Arc::from(
                panic_payload_message(&payload),
            )),
        }
    }

    pub(super) fn request_cancellation(&self) {
        let mut state = self.lock_state();
        state.cancellation_requested = true;
    }

    pub(super) fn lock_state(&self) -> MutexGuard<'_, TaskRecordState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
