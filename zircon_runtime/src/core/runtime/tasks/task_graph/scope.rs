use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::Duration;

use super::super::job_scheduler::JobExecutionOutcome;
use super::super::{JobHandle, JobScheduler, TaskCancellationPolicy, TaskDescriptor, TaskId};
use super::admission::TaskGraphAdmissionError;
use super::engine_task_graph::EngineTaskGraphInner;
use super::lease::TaskGraphClientLease;
use super::scope_model::{TaskGraphScopeCensus, TaskGraphScopeDescriptor};
use super::scope_registration::TaskGraphScopeRegistration;
use super::task_handle::TaskHandle;

mod cancellation;
mod task_admission;

pub use cancellation::TaskCancellationToken;
use task_admission::TaskAdmission;

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
    tasks: HashMap<TaskId, ScopeTask>,
}

struct ScopeTask {
    completion: JobHandle,
    running: bool,
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
        let scheduler = graph.scheduler_for(descriptor.kind);
        self.submit_with_scheduler(&graph, &scheduler, descriptor, task)
    }

    /// Uses the supplied execution owner while this scope accounts for admission and cancellation.
    /// Original runtime refusal takes precedence over a missing or closed target scope.
    pub fn submit_on_scheduler(
        &self,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let (submission, original_graph) = scheduler.acquire_task_submission()?;
        if descriptor.kind != scheduler.pool_kind() {
            return Err(TaskGraphAdmissionError::SchedulerKindMismatch {
                descriptor: descriptor.kind,
                scheduler: scheduler.pool_kind(),
            });
        }
        let target_graph = self
            .graph
            .upgrade()
            .ok_or(TaskGraphAdmissionError::RuntimeUnavailable)?;
        target_graph.ensure_admission_open()?;
        let completion = match original_graph.as_ref() {
            Some(graph) => graph.pending_scheduler_completion(scheduler, descriptor, 0),
            None => scheduler.pending_task_completion(descriptor, 0),
        };
        self.inner.admit(completion.clone())?;
        let task_id = completion.descriptor().id;
        let scope = Arc::clone(&self.inner);
        let retirement = Arc::clone(&self.inner);
        let completion = scheduler.schedule_existing_with_submission_and_post_terminal(
            completion,
            submission,
            move || scope.run(task_id, task),
            Some(Box::new(move |outcome| {
                retirement.retire(task_id, &outcome)
            })),
        );
        Ok(TaskHandle::new(completion, Some(Arc::clone(&self.inner))))
    }

    /// Schedules descriptor-led work after canonical task handles without requiring a
    /// separately-provided scheduler facade. The graph supplies the physical owner and
    /// callback dispatcher, so this route cannot create a private worker or callback lane.
    pub fn submit_after(
        &self,
        dependencies: &[TaskHandle],
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let graph = self
            .graph
            .upgrade()
            .ok_or(TaskGraphAdmissionError::RuntimeUnavailable)?;
        let scheduler = graph.scheduler_for(descriptor.kind);
        self.submit_after_with_scheduler(&graph, &scheduler, dependencies, descriptor, task)
    }

    fn submit_with_scheduler(
        &self,
        graph: &EngineTaskGraphInner,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let admission = self.admit(graph, scheduler, descriptor, 0)?;
        let task_id = admission.completion.descriptor().id;
        let scope = Arc::clone(&self.inner);
        let scope_for_retirement = Arc::clone(&self.inner);
        let completion = scheduler.schedule_existing_with_submission_and_post_terminal(
            admission.completion,
            admission.submission,
            move || scope.run(task_id, task),
            Some(Box::new(move |outcome| {
                scope_for_retirement.retire(task_id, &outcome);
            })),
        );
        Ok(TaskHandle::new(completion, Some(Arc::clone(&self.inner))))
    }

    /// Schedules scoped work after all dependencies complete successfully.
    ///
    /// A failed dependency retires the queued record without launching user code.
    // 依赖先校验属于同一 graph owner，再由 dependency leases 保活至终态；前置依赖失败时 scheduler 走不启动用户闭包的退休钩子。
    fn submit_after_with_scheduler(
        &self,
        graph: &EngineTaskGraphInner,
        scheduler: &JobScheduler,
        dependencies: &[TaskHandle],
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        if dependencies
            .iter()
            .any(|dependency| !dependency.belongs_to_graph(graph.owner_identity()))
        {
            return Err(TaskGraphAdmissionError::DependencyOwnerMismatch {
                owner: self.inner.descriptor.owner.clone(),
            });
        }
        let admission = self.admit(graph, scheduler, descriptor, dependencies.len())?;
        let task_id = admission.completion.descriptor().id;
        let scope_for_task = Arc::clone(&self.inner);
        let scope_for_retirement = Arc::clone(&self.inner);
        let scope_for_prelaunch_terminal = Arc::clone(&self.inner);
        let mut dependency_fences = Vec::with_capacity(dependencies.len());
        dependency_fences.extend(
            dependencies
                .iter()
                .map(|dependency| dependency.completion.clone()),
        );
        let dependency_leases = Arc::new(dependencies.to_vec());
        let dependency_leases_for_task = Arc::clone(&dependency_leases);
        let dependency_leases_for_prelaunch_terminal = Arc::clone(&dependency_leases);
        let completion = scheduler.schedule_after_existing_with_submission_and_hooks(
            admission.completion,
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
            move |outcome| {
                scope_for_retirement.retire(task_id, &outcome);
            },
        );
        Ok(TaskHandle::new(completion, Some(Arc::clone(&self.inner))))
    }

    fn admit(
        &self,
        graph: &EngineTaskGraphInner,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        remaining_dependencies: usize,
    ) -> Result<TaskAdmission, TaskGraphAdmissionError> {
        let submission = graph.acquire_worker_submission(descriptor.kind)?;
        let completion =
            graph.pending_scheduler_completion(scheduler, descriptor, remaining_dependencies);
        self.inner.admit(completion.clone())?;
        Ok(TaskAdmission {
            completion,
            submission,
        })
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
        for task in state.tasks.values() {
            if task.completion.descriptor().cancellation_policy
                == TaskCancellationPolicy::CancelOnDrop
            {
                task.completion.task_node_ref().request_cancellation();
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

    fn admit(&self, completion: JobHandle) -> Result<(), TaskGraphAdmissionError> {
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
        let task_id = completion.descriptor().id;
        if state.tasks.contains_key(&task_id) {
            return Err(TaskGraphAdmissionError::TaskIdAlreadyActive {
                owner: self.descriptor.owner.clone(),
                id: task_id.raw(),
            });
        }
        state.tasks.insert(
            task_id,
            ScopeTask {
                completion,
                running: false,
            },
        );
        state.submitted = state.submitted.saturating_add(1);
        state.queued += 1;
        Ok(())
    }

    fn run(
        &self,
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
                let message = panic_payload_message(payload);
                self.finish(task_id, Some(message))
            }
        }
    }

    fn begin(&self, task_id: TaskId) -> Option<TaskCancellationToken> {
        let mut scope = self.lock_state();
        let completion = scope.tasks.get(&task_id)?.completion.clone();
        let node = completion.task_node();
        let cancellation_requested = node.lock_inner().cancellation_requested;
        if cancellation_requested {
            return None;
        }
        scope.queued = scope.queued.saturating_sub(1);
        scope.running += 1;
        scope.tasks.get_mut(&task_id)?.running = true;
        Some(TaskCancellationToken { node })
    }

    fn finish(&self, task_id: TaskId, failure_message: Option<String>) -> JobExecutionOutcome {
        let scope = self.lock_state();
        let Some(completion) = scope
            .tasks
            .get(&task_id)
            .map(|task| task.completion.clone())
        else {
            return JobExecutionOutcome::Cancelled;
        };
        let task = completion.task_node();
        let task = task.lock_inner();
        if let Some(message) = failure_message {
            JobExecutionOutcome::Panicked(Arc::from(message))
        } else if task.cancellation_acknowledged {
            JobExecutionOutcome::Cancelled
        } else {
            JobExecutionOutcome::Completed
        }
    }

    /// Retires a scope record only after its JobHandle has published the terminal state.
    /// Keeping the record active until this point makes quiescence and TaskId admission
    /// observe the same completion boundary as the public handle.
    fn retire(&self, task_id: TaskId, outcome: &JobExecutionOutcome) {
        let mut scope = self.lock_state();
        let Some(task) = scope.tasks.remove(&task_id) else {
            return;
        };
        if task.running {
            scope.running = scope.running.saturating_sub(1);
        } else {
            scope.queued = scope.queued.saturating_sub(1);
        }
        match outcome {
            JobExecutionOutcome::Panicked(_) => {
                scope.failed = scope.failed.saturating_add(1);
            }
            JobExecutionOutcome::Cancelled => {
                scope.cancelled = scope.cancelled.saturating_add(1);
            }
            JobExecutionOutcome::Completed => {
                scope.completed = scope.completed.saturating_add(1);
            }
        }
        self.quiescent.notify_all();
    }

    fn retire_prelaunch_failure(&self, task_id: TaskId, cancelled: bool) {
        let mut scope = self.lock_state();
        if scope.tasks.remove(&task_id).is_none() {
            return;
        }
        scope.queued = scope.queued.saturating_sub(1);
        if cancelled {
            scope.cancelled = scope.cancelled.saturating_add(1);
        } else {
            scope.failed = scope.failed.saturating_add(1);
        }
        self.quiescent.notify_all();
    }

    fn fail_without_execution(&self, task_id: TaskId, _message: String) {
        self.retire_prelaunch_failure(task_id, false);
    }

    fn cancel_without_execution(&self, task_id: TaskId) {
        self.retire_prelaunch_failure(task_id, true);
    }

    fn lock_state(&self) -> MutexGuard<'_, TaskGraphScopeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let payload = match payload.downcast::<String>() {
        Ok(message) => return *message,
        Err(payload) => payload,
    };
    match payload.downcast::<&str>() {
        Ok(message) => (*message).to_owned(),
        Err(_) => "non-string panic payload".to_owned(),
    }
}

#[cfg(test)]
#[path = "scope/tests/panic_payload.rs"]
mod panic_payload_tests;

#[cfg(test)]
#[path = "scope/tests/cases.rs"]
mod tests;
