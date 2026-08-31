use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::Duration;

use super::super::job_scheduler::JobExecutionOutcome;
use super::super::{JobScheduler, TaskCancellationPolicy, TaskDescriptor, TaskId};
use super::admission::TaskGraphAdmissionError;
use super::engine_task_graph::EngineTaskGraphInner;
use super::lease::TaskGraphClientLease;
use super::scope_model::{TaskGraphScopeCensus, TaskGraphScopeDescriptor};
use super::scope_registration::TaskGraphScopeRegistration;
use super::task_handle::{TaskHandle, TaskRecord};

mod cancellation;
mod scheduler_admission;

pub use cancellation::TaskCancellationToken;
use scheduler_admission::SchedulerTaskAdmission;

pub(super) struct TaskGraphScopeInner {
    descriptor: TaskGraphScopeDescriptor,
    _registration: TaskGraphScopeRegistration,
    state: Mutex<TaskGraphScopeState>,
    quiescent: Condvar,
}

struct TaskGraphScopeState {
    accepting: bool,
    submitted: u64,
    queued: usize,
    running: usize,
    completed: u64,
    failed: u64,
    cancelled: u64,
    tasks: HashMap<TaskId, Arc<TaskRecord>>,
}

/// A subsystem-owned gate for task submission and shutdown accounting.
pub struct TaskGraphScope {
    inner: Arc<TaskGraphScopeInner>,
    graph: Weak<EngineTaskGraphInner>,
    scope_lease: Arc<TaskGraphClientLease>,
}

impl TaskGraphScope {
    pub(super) fn new(inner: Arc<TaskGraphScopeInner>, graph: Weak<EngineTaskGraphInner>) -> Self {
        Self {
            inner,
            graph,
            scope_lease: TaskGraphClientLease::new(),
        }
    }

    pub fn descriptor(&self) -> &TaskGraphScopeDescriptor {
        &self.inner.descriptor
    }

    pub fn census(&self) -> TaskGraphScopeCensus {
        self.inner.census()
    }

    pub fn close_admission(&self) {
        self.inner.close_admission();
    }

    /// Waits for admitted work to terminate; close admission first for a stable drain.
    pub fn wait_until_quiescent(&self, timeout: Duration) -> bool {
        self.inner.wait_until_quiescent(timeout)
    }

    pub fn submit(
        &self,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let graph = self
            .graph
            .upgrade()
            .ok_or(TaskGraphAdmissionError::RuntimeUnavailable)?;
        let submission = graph.acquire_worker_submission()?;
        let record = self.inner.admit(descriptor)?;
        let task_id = record.descriptor.id;
        let scope = Arc::clone(&self.inner);
        let completion = graph.pending_completion();
        let completion_for_task = completion.clone();
        submission.spawn(move || {
            completion_for_task.mark_running();
            match scope.run(task_id, task) {
                JobExecutionOutcome::Cancelled => completion_for_task.mark_cancelled(),
                JobExecutionOutcome::Completed => completion_for_task.mark_complete(),
                JobExecutionOutcome::Panicked(message) => {
                    completion_for_task.mark_panicked(message)
                }
            }
        });
        Ok(TaskHandle {
            record,
            scope: Some(Arc::clone(&self.inner)),
            completion,
            handle_lease: TaskGraphClientLease::new(),
        })
    }

    /// Schedules through a scheduler that shares this graph's worker owner while
    /// retaining scope admission, cancellation, and drain ownership.
    pub fn schedule(
        &self,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let admission = self.admit_for_scheduler(scheduler, descriptor)?;
        let task_id = admission.record.descriptor.id;
        let scope = Arc::clone(&self.inner);
        let completion = scheduler
            .schedule_with_submission(admission.submission, move || scope.run(task_id, task));
        Ok(TaskHandle {
            record: admission.record,
            scope: Some(Arc::clone(&self.inner)),
            completion,
            handle_lease: TaskGraphClientLease::new(),
        })
    }

    /// Schedules scoped work after all dependencies complete successfully.
    ///
    /// A failed dependency retires the queued record without launching user code.
    pub fn schedule_after(
        &self,
        scheduler: &JobScheduler,
        dependencies: &[TaskHandle],
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let admission = self.admit_for_scheduler(scheduler, descriptor)?;
        let task_id = admission.record.descriptor.id;
        let scope_for_task = Arc::clone(&self.inner);
        let scope_for_prelaunch_terminal = Arc::clone(&self.inner);
        let dependency_fences = dependencies
            .iter()
            .map(|dependency| dependency.completion.clone())
            .collect::<Vec<_>>();
        let dependency_leases = Arc::new(dependencies.to_vec());
        let dependency_leases_for_task = Arc::clone(&dependency_leases);
        let dependency_leases_for_prelaunch_terminal = Arc::clone(&dependency_leases);
        let completion = scheduler.schedule_after_with_submission_and_prelaunch_terminal(
            &dependency_fences,
            admission.submission,
            move || {
                drop(dependency_leases_for_task);
                scope_for_task.run(task_id, task)
            },
            move |kind, message| {
                drop(dependency_leases_for_prelaunch_terminal);
                match kind {
                    super::super::TaskDiagnosticKind::Cancelled => {
                        scope_for_prelaunch_terminal.cancel_without_execution(task_id);
                    }
                    super::super::TaskDiagnosticKind::Panicked => {
                        scope_for_prelaunch_terminal.fail_without_execution(
                            task_id,
                            format!("dependency prevented scoped task execution: {message}"),
                        );
                    }
                }
            },
        );
        Ok(TaskHandle {
            record: admission.record,
            scope: Some(Arc::clone(&self.inner)),
            completion,
            handle_lease: TaskGraphClientLease::new(),
        })
    }

    fn admit_for_scheduler(
        &self,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
    ) -> Result<SchedulerTaskAdmission, TaskGraphAdmissionError> {
        let graph = self
            .graph
            .upgrade()
            .ok_or(TaskGraphAdmissionError::RuntimeUnavailable)?;
        if !graph.shares_worker_owner_with(scheduler) {
            return Err(TaskGraphAdmissionError::SchedulerOwnerMismatch {
                owner: self.inner.descriptor.owner.clone(),
            });
        }
        let submission = graph.acquire_worker_submission()?;
        let record = self.inner.admit(descriptor)?;
        Ok(SchedulerTaskAdmission { record, submission })
    }
}

impl Clone for TaskGraphScope {
    fn clone(&self) -> Self {
        self.scope_lease.retain();
        Self {
            inner: Arc::clone(&self.inner),
            graph: self.graph.clone(),
            scope_lease: Arc::clone(&self.scope_lease),
        }
    }
}

impl Drop for TaskGraphScope {
    fn drop(&mut self) {
        if self.scope_lease.release() {
            self.inner.close_admission();
        }
    }
}

impl TaskGraphScopeInner {
    pub(super) fn new(
        descriptor: TaskGraphScopeDescriptor,
        graph: Weak<EngineTaskGraphInner>,
        scope_id: u64,
    ) -> Self {
        Self {
            descriptor,
            _registration: TaskGraphScopeRegistration::new(graph, scope_id),
            state: Mutex::new(TaskGraphScopeState {
                accepting: true,
                submitted: 0,
                queued: 0,
                running: 0,
                completed: 0,
                failed: 0,
                cancelled: 0,
                tasks: HashMap::new(),
            }),
            quiescent: Condvar::new(),
        }
    }

    pub(super) fn close_admission(&self) {
        let mut state = self.lock_state();
        if !state.accepting {
            return;
        }
        state.accepting = false;
        // Workers take the scope lock before a task lock. Mark cancellation
        // under the same lock so a queued CancelOnDrop task cannot begin in
        // the gap between closing admission and its cancellation request.
        for record in state.tasks.values() {
            if record.descriptor.cancellation_policy == TaskCancellationPolicy::CancelOnDrop {
                record.request_cancellation();
            }
        }
    }

    pub(super) fn wait_until_quiescent(&self, timeout: Duration) -> bool {
        let mut state = self.lock_state();
        if state.queued == 0 && state.running == 0 {
            return true;
        }
        let (state_after_wait, _) = self
            .quiescent
            .wait_timeout_while(state, timeout, |state| {
                state.queued != 0 || state.running != 0
            })
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state = state_after_wait;
        state.queued == 0 && state.running == 0
    }

    pub(super) fn census(&self) -> TaskGraphScopeCensus {
        let state = self.lock_state();
        TaskGraphScopeCensus {
            owner: self.descriptor.owner.clone(),
            task_capacity: self.descriptor.task_capacity,
            accepting: state.accepting,
            submitted: state.submitted,
            queued: state.queued,
            running: state.running,
            completed: state.completed,
            failed: state.failed,
            cancelled: state.cancelled,
        }
    }

    fn admit(
        &self,
        descriptor: TaskDescriptor,
    ) -> Result<Arc<TaskRecord>, TaskGraphAdmissionError> {
        let mut state = self.lock_state();
        if !state.accepting {
            return Err(TaskGraphAdmissionError::ScopeClosed {
                owner: self.descriptor.owner.clone(),
            });
        }
        if state.tasks.len() >= self.descriptor.task_capacity {
            return Err(TaskGraphAdmissionError::ScopeCapacityReached {
                owner: self.descriptor.owner.clone(),
                capacity: self.descriptor.task_capacity,
            });
        }
        let task_id = descriptor.id;
        if state.tasks.contains_key(&task_id) {
            return Err(TaskGraphAdmissionError::TaskIdAlreadyActive {
                owner: self.descriptor.owner.clone(),
                id: task_id.raw(),
            });
        }
        let record = Arc::new(TaskRecord::new(descriptor));
        state.tasks.insert(task_id, Arc::clone(&record));
        state.submitted = state.submitted.saturating_add(1);
        state.queued += 1;
        Ok(record)
    }

    fn run(
        self: Arc<Self>,
        task_id: TaskId,
        task: impl FnOnce(TaskCancellationToken),
    ) -> JobExecutionOutcome {
        let Some(token) = self.begin(task_id) else {
            return JobExecutionOutcome::Cancelled;
        };
        let result = catch_unwind(AssertUnwindSafe(|| task(token)));
        match result {
            Ok(()) => self.finish(task_id, None),
            Err(payload) => {
                let message = panic_payload_message(&payload);
                self.finish(task_id, Some(message))
            }
        }
    }

    fn begin(&self, task_id: TaskId) -> Option<TaskCancellationToken> {
        let mut scope = self.lock_state();
        let record = scope.tasks.get(&task_id)?.clone();
        let cancellation_requested = record.lock_state().cancellation_requested;
        if cancellation_requested {
            scope.tasks.remove(&task_id);
            scope.queued = scope.queued.saturating_sub(1);
            scope.cancelled = scope.cancelled.saturating_add(1);
            self.quiescent.notify_all();
            return None;
        }
        scope.queued = scope.queued.saturating_sub(1);
        scope.running += 1;
        Some(TaskCancellationToken { record })
    }

    fn finish(&self, task_id: TaskId, failure_message: Option<String>) -> JobExecutionOutcome {
        let mut scope = self.lock_state();
        let Some(record) = scope.tasks.remove(&task_id) else {
            return JobExecutionOutcome::Cancelled;
        };
        let mut task = record.lock_state();
        let outcome = if let Some(message) = failure_message {
            scope.failed = scope.failed.saturating_add(1);
            JobExecutionOutcome::Panicked(Arc::from(message))
        } else if task.cancellation_acknowledged {
            scope.cancelled = scope.cancelled.saturating_add(1);
            JobExecutionOutcome::Cancelled
        } else {
            scope.completed = scope.completed.saturating_add(1);
            JobExecutionOutcome::Completed
        };
        scope.running = scope.running.saturating_sub(1);
        self.quiescent.notify_all();
        outcome
    }

    fn fail_without_execution(&self, task_id: TaskId, _message: String) {
        let mut scope = self.lock_state();
        if scope.tasks.remove(&task_id).is_none() {
            return;
        }
        scope.queued = scope.queued.saturating_sub(1);
        scope.failed = scope.failed.saturating_add(1);
        self.quiescent.notify_all();
    }

    fn cancel_without_execution(&self, task_id: TaskId) {
        let mut scope = self.lock_state();
        if scope.tasks.remove(&task_id).is_none() {
            return;
        }
        scope.queued = scope.queued.saturating_sub(1);
        scope.cancelled = scope.cancelled.saturating_add(1);
        self.quiescent.notify_all();
    }

    fn lock_state(&self) -> MutexGuard<'_, TaskGraphScopeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

#[cfg(test)]
#[path = "scope/tests.rs"]
mod tests;
