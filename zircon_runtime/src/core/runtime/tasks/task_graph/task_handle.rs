use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use super::super::{
    JobHandle, JobScheduler, TaskCancellationPolicy, TaskDescriptor, TaskNode, TaskStatus,
};
use super::scope::{panic_payload_message, TaskCancellationToken, TaskGraphScopeInner};
use super::scope_model::TaskGraphScopeCensus;

/// Canonical handle for descriptor-led work admitted to the Runtime task owner.
pub struct TaskHandle {
    pub(super) scope: Option<Arc<TaskGraphScopeInner>>,
    pub(super) completion: JobHandle,
}

impl TaskHandle {
    pub(crate) fn schedule_detached(
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Self {
        let completion = scheduler.pending_task_completion(descriptor, 0);
        let node_for_task = completion.task_node();
        scheduler.schedule_existing_with_outcome(completion.clone(), move || {
            node_for_task.run_detached(task)
        });
        Self::new(completion, None)
    }

    pub(crate) fn try_schedule_detached(
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) + Send + 'static,
    ) -> Result<Self, super::TaskGraphAdmissionError> {
        let (submission, graph) = scheduler.acquire_task_submission()?;
        if descriptor.kind != scheduler.pool_kind() {
            return Err(super::TaskGraphAdmissionError::SchedulerKindMismatch {
                descriptor: descriptor.kind,
                scheduler: scheduler.pool_kind(),
            });
        }
        let completion = match graph.as_ref() {
            Some(graph) => graph.pending_scheduler_completion(scheduler, descriptor, 0),
            None => scheduler.pending_task_completion(descriptor, 0),
        };
        let node = completion.task_node();
        let completion =
            scheduler.schedule_existing_with_submission(completion, submission, move || {
                node.run_detached(task)
            });
        Ok(Self::new(completion, None))
    }

    pub(crate) fn completed(descriptor: TaskDescriptor) -> Self {
        Self::new(JobHandle::completed_task(descriptor), None)
    }

    pub(super) fn new(completion: JobHandle, scope: Option<Arc<TaskGraphScopeInner>>) -> Self {
        completion.task_node_ref().retain_public_handle();
        Self { scope, completion }
    }

    pub(super) fn belongs_to_graph(&self, graph_owner: &Arc<()>) -> bool {
        self.completion.belongs_to_graph(graph_owner)
    }

    pub fn descriptor(&self) -> &TaskDescriptor {
        self.completion.descriptor()
    }

    pub fn status(&self) -> TaskStatus {
        self.completion.task_status(self.descriptor().id)
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

    /// Waits for every canonical task handle to reach a terminal state before
    /// propagating a task panic. Cancellation remains a normal wait completion.
    pub fn wait_all(handles: &[Self]) {
        match handles {
            [] => return,
            [handle] => handle.wait(),
            _ => {}
        }
        let completions = handles
            .iter()
            .map(|handle| handle.completion.clone())
            .collect::<Vec<_>>();
        JobHandle::combine(&completions).wait();
    }

    pub fn on_terminal(&self, observer: impl FnOnce() + Send + 'static) {
        self.completion.on_terminal(observer);
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.completion
            .task_node_ref()
            .lock_inner()
            .cancellation_requested
    }

    /// Requests cooperative cancellation. Running work observes the request
    /// through its token; queued work converts to `Cancelled` when its worker
    /// closure is reached, preserving queue-drain accounting.
    pub fn request_cancellation(&self) {
        if !self.completion.is_complete() {
            self.completion.task_node_ref().request_cancellation();
        }
    }

    pub fn scope_census(&self) -> TaskGraphScopeCensus {
        self.scope
            .as_ref()
            .expect("public task handles are scope-owned")
            .census()
    }

    #[cfg(test)]
    pub(crate) fn task_node_identity(&self) -> usize {
        self.completion.task_node_identity()
    }

    #[cfg(test)]
    pub(crate) fn completion_node_identity(&self) -> usize {
        self.completion.task_node_identity()
    }

    #[cfg(test)]
    pub(crate) fn graph_owner_weak_count(&self) -> usize {
        self.completion.graph_owner_weak_count()
    }
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        self.completion.task_node_ref().retain_public_handle();
        Self {
            scope: self.scope.clone(),
            completion: self.completion.clone(),
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

// 只有公开句柄计数归零才触发 CancelOnDrop；内部 completion 或依赖引用不会被误当成用户放弃任务。
impl Drop for TaskHandle {
    fn drop(&mut self) {
        if self.completion.task_node_ref().release_public_handle()
            && self.descriptor().cancellation_policy == TaskCancellationPolicy::CancelOnDrop
        {
            self.request_cancellation();
        }
    }
}

impl TaskNode {
    // detached 任务没有 scope census，仍通过同一个 TaskNode 发布取消确认、panic 和完成状态，供句柄等待及依赖观察共享。
    fn run_detached(
        self: Arc<Self>,
        task: impl FnOnce(TaskCancellationToken),
    ) -> super::super::job_scheduler::JobExecutionOutcome {
        {
            let state = self.lock_inner();
            if state.cancellation_requested {
                return super::super::job_scheduler::JobExecutionOutcome::Cancelled;
            }
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            task(TaskCancellationToken {
                node: Arc::clone(&self),
            });
        }));
        let state = self.lock_inner();
        match result {
            Ok(()) if state.cancellation_acknowledged => {
                super::super::job_scheduler::JobExecutionOutcome::Cancelled
            }
            Ok(()) => super::super::job_scheduler::JobExecutionOutcome::Completed,
            Err(payload) => super::super::job_scheduler::JobExecutionOutcome::Panicked(Arc::from(
                panic_payload_message(payload),
            )),
        }
    }
}
