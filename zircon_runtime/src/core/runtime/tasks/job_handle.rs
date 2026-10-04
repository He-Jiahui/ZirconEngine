use std::fmt;
use std::sync::{Arc, Weak};
use std::time::Duration;

use super::callback_dispatcher::{TaskCallback, TaskCallbackDispatcher};
use super::pool::{assist_current_thread_once, TaskPoolYield};
use super::{JobSchedulerDiagnosticsState, TaskDescriptor, TaskId, TaskState, TaskStatus};

mod task_node;

pub(crate) use task_node::TaskNode;
use task_node::TaskNodeState;

type JobContinuation = TaskCallback;
type JobTerminalObserver = TaskCallback;
const WORKER_WAIT_IDLE_PARK: Duration = Duration::from_millis(1);

#[derive(Clone)]
pub struct JobHandle {
    node: Arc<TaskNode>,
    callback_dispatcher: TaskCallbackDispatcher,
    wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
}

impl JobHandle {
    pub(super) fn pending_with_dependencies(remaining_dependencies: usize) -> Self {
        Self::pending_with_wait_diagnostics(
            remaining_dependencies,
            None,
            default_callback_dispatcher(),
        )
    }

    pub(super) fn pending_with_scheduler_diagnostics(
        remaining_dependencies: usize,
        wait_diagnostics: Arc<JobSchedulerDiagnosticsState>,
        callback_dispatcher: TaskCallbackDispatcher,
    ) -> Self {
        Self::pending_with_wait_diagnostics(
            remaining_dependencies,
            Some(wait_diagnostics),
            callback_dispatcher,
        )
    }

    pub(super) fn pending_with_callback_dispatcher(
        remaining_dependencies: usize,
        callback_dispatcher: TaskCallbackDispatcher,
    ) -> Self {
        Self::pending_with_wait_diagnostics(remaining_dependencies, None, callback_dispatcher)
    }

    fn pending_with_wait_diagnostics(
        remaining_dependencies: usize,
        wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
        callback_dispatcher: TaskCallbackDispatcher,
    ) -> Self {
        Self::pending_node(
            None,
            remaining_dependencies,
            wait_diagnostics,
            callback_dispatcher,
            None,
        )
    }

    pub(super) fn pending_task_with_scheduler_diagnostics(
        descriptor: TaskDescriptor,
        remaining_dependencies: usize,
        wait_diagnostics: Arc<JobSchedulerDiagnosticsState>,
        callback_dispatcher: TaskCallbackDispatcher,
    ) -> Self {
        Self::pending_node(
            Some(descriptor),
            remaining_dependencies,
            Some(wait_diagnostics),
            callback_dispatcher,
            None,
        )
    }

    pub(super) fn pending_graph_task_with_scheduler_diagnostics(
        descriptor: TaskDescriptor,
        remaining_dependencies: usize,
        wait_diagnostics: Arc<JobSchedulerDiagnosticsState>,
        callback_dispatcher: TaskCallbackDispatcher,
        graph_owner: Weak<()>,
    ) -> Self {
        Self::pending_node(
            Some(descriptor),
            remaining_dependencies,
            Some(wait_diagnostics),
            callback_dispatcher,
            Some(graph_owner),
        )
    }

    fn pending_node(
        descriptor: Option<TaskDescriptor>,
        remaining_dependencies: usize,
        wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
        callback_dispatcher: TaskCallbackDispatcher,
        graph_owner: Option<Weak<()>>,
    ) -> Self {
        Self {
            node: Arc::new(TaskNode::new(
                descriptor,
                remaining_dependencies,
                graph_owner,
            )),
            callback_dispatcher,
            wait_diagnostics,
        }
    }

    pub fn completed() -> Self {
        let handle = Self::pending_with_dependencies(0);
        handle.mark_complete();
        handle
    }

    pub(super) fn completed_task(descriptor: TaskDescriptor) -> Self {
        let handle = Self::pending_node(
            Some(descriptor),
            0,
            None,
            default_callback_dispatcher(),
            None,
        );
        handle.mark_complete();
        handle
    }

    pub fn combine(handles: &[JobHandle]) -> Self {
        Self::combine_with_wait_diagnostics(
            handles,
            handles
                .iter()
                .find_map(|handle| handle.wait_diagnostics.clone()),
        )
    }

    pub(super) fn combine_with_scheduler_diagnostics(
        handles: &[JobHandle],
        wait_diagnostics: Arc<JobSchedulerDiagnosticsState>,
    ) -> Self {
        Self::combine_with_wait_diagnostics(handles, Some(wait_diagnostics))
    }

    fn combine_with_wait_diagnostics(
        handles: &[JobHandle],
        wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
    ) -> Self {
        if handles.is_empty() {
            return Self::completed();
        }

        let callback_dispatcher = handles
            .first()
            .map(|handle| handle.callback_dispatcher.clone())
            .unwrap_or_else(default_callback_dispatcher);
        let combined = Self::pending_with_wait_diagnostics(
            handles.len(),
            wait_diagnostics,
            callback_dispatcher,
        );
        for handle in handles {
            let terminal = {
                let mut inner = handle.node.lock_inner();
                if inner.lifecycle.is_terminal() {
                    Some((
                        inner.panic_message.clone(),
                        inner.lifecycle == TaskState::Cancelled,
                    ))
                } else {
                    let handle_for_callback = handle.clone();
                    let combined_for_callback = combined.clone();
                    inner.dependents.push(Box::new(move || {
                        combined_for_callback.combined_dependency_completed(
                            handle_for_callback.panic_message(),
                            handle_for_callback.is_cancelled(),
                        );
                    }));
                    None
                }
            };
            if let Some((panic_message, is_cancelled)) = terminal {
                combined.combined_dependency_completed(panic_message, is_cancelled);
            }
        }
        combined
    }

    pub fn is_complete(&self) -> bool {
        self.node.lock_inner().lifecycle.is_terminal()
    }

    pub fn is_cancelled(&self) -> bool {
        let inner = self.node.lock_inner();
        inner.lifecycle == TaskState::Cancelled
    }

    pub fn terminal_state(&self) -> Option<TaskState> {
        let inner = self.node.lock_inner();
        inner.lifecycle.is_terminal().then_some(inner.lifecycle)
    }

    pub(super) fn task_status(&self, id: TaskId) -> TaskStatus {
        let inner = self.node.lock_inner();
        TaskStatus {
            id,
            state: inner.lifecycle,
            failure_message: inner
                .panic_message
                .as_ref()
                .filter(|_| inner.lifecycle == TaskState::Failed)
                .map(|message| message.to_string()),
        }
    }

    /// Runs `observer` once after this handle reaches any terminal state.
    ///
    /// Scheduler-backed handles deliver through their owner dispatcher after all dependency
    /// continuations are released. Standalone handles and registrations made after the owner
    /// stops use the same ordered callback queue with inline delivery. `wait()` synchronizes
    /// terminal state, not observer completion.
    pub fn on_terminal(&self, observer: impl FnOnce() + Send + 'static) {
        let observer: JobTerminalObserver = Box::new(observer);
        let observers = {
            let mut inner = self.node.lock_inner();
            inner.terminal_observers.push(observer);
            if inner.lifecycle.is_terminal() && inner.dependency_continuations_published {
                TaskNode::take_terminal_observer_batch(&mut inner)
            } else {
                None
            }
        };

        if let Some(observers) = observers {
            dispatch_terminal_observer_batch(
                Arc::clone(&self.node),
                self.callback_dispatcher.clone(),
                observers,
            );
        }
    }

    /// Returns the number of terminal observers whose panics were contained for this handle.
    pub fn terminal_observer_panic_count(&self) -> usize {
        self.node.terminal_observer_panic_count()
    }

    pub fn wait(&self) {
        let started_at = self
            .wait_diagnostics
            .as_ref()
            .and_then(|diagnostics| diagnostics.explicit_wait_started_at());
        let panic_message = self.wait_for_terminal();
        if let Some(diagnostics) = &self.wait_diagnostics {
            diagnostics.record_explicit_wait(started_at);
        }
        if let Some(panic_message) = panic_message {
            panic!("job task panicked: {}", panic_message.as_ref());
        }
    }

    fn wait_for_terminal(&self) -> Option<Arc<str>> {
        let mut inner = self.node.lock_inner();
        while !inner.lifecycle.is_terminal() {
            drop(inner);
            if let Some(result) = assist_current_thread_once() {
                inner = self.node.lock_inner();
                if !inner.lifecycle.is_terminal() && result == TaskPoolYield::Idle {
                    inner = self.node.wait_inner_timeout(inner, WORKER_WAIT_IDLE_PARK);
                }
            } else {
                inner = self.node.lock_inner();
                if !inner.lifecycle.is_terminal() {
                    inner = self.node.wait_inner(inner);
                }
            }
        }
        inner.panic_message.clone()
    }

    pub(super) fn mark_complete(&self) {
        self.mark_terminal(None, false);
    }

    pub(super) fn mark_running(&self) {
        let mut inner = self.node.lock_inner();
        if inner.lifecycle == TaskState::Pending {
            inner.lifecycle = TaskState::Running;
        }
    }

    pub(super) fn mark_panicked(&self, panic_message: impl Into<Arc<str>>) {
        self.mark_terminal(Some(panic_message.into()), false);
    }

    pub(super) fn mark_cancelled(&self) {
        self.mark_terminal(None, true);
    }

    pub(super) fn panic_message(&self) -> Option<Arc<str>> {
        self.node.lock_inner().panic_message.clone()
    }

    fn mark_terminal(&self, panic_message: Option<Arc<str>>, is_cancelled: bool) {
        let dependents = {
            let mut inner = self.node.lock_inner();
            if inner.lifecycle.is_terminal() {
                return;
            }
            inner.panic_message = panic_message;
            inner.is_cancelled = is_cancelled;
            inner.lifecycle = if inner.panic_message.is_some() {
                TaskState::Failed
            } else if inner.is_cancelled {
                TaskState::Cancelled
            } else {
                TaskState::Completed
            };
            std::mem::take(&mut inner.dependents)
        };

        self.dispatch_terminal(dependents);
    }

    pub(super) fn add_dependent(&self, dependent: JobContinuation) -> bool {
        let mut inner = self.node.lock_inner();
        if inner.lifecycle.is_terminal() {
            false
        } else {
            inner.dependents.push(dependent);
            true
        }
    }

    pub(super) fn dependency_completed(&self) -> bool {
        let mut inner = self.node.lock_inner();
        if inner.lifecycle.is_terminal() || inner.remaining_dependencies == 0 {
            return false;
        }

        inner.remaining_dependencies -= 1;
        inner.remaining_dependencies == 0 && !inner.lifecycle.is_terminal()
    }

    // 组合句柄在同一状态锁内累计首个 panic、任一取消和剩余依赖数，只有所有输入终结后才发布组合终态。
    fn combined_dependency_completed(&self, panic_message: Option<Arc<str>>, is_cancelled: bool) {
        let dependents = {
            let mut inner = self.node.lock_inner();
            if inner.lifecycle.is_terminal() || inner.remaining_dependencies == 0 {
                return;
            }
            if inner.panic_message.is_none() {
                inner.panic_message = panic_message;
            }
            inner.is_cancelled |= is_cancelled;
            inner.remaining_dependencies -= 1;
            if inner.remaining_dependencies != 0 {
                return;
            }
            inner.lifecycle = if inner.panic_message.is_some() {
                TaskState::Failed
            } else if inner.is_cancelled {
                TaskState::Cancelled
            } else {
                TaskState::Completed
            };
            std::mem::take(&mut inner.dependents)
        };

        self.dispatch_terminal(dependents);
    }

    fn dispatch_terminal(&self, dependents: Vec<JobContinuation>) {
        self.node.notify_terminal();
        let node = Arc::clone(&self.node);
        let dispatcher = self.callback_dispatcher.clone();
        self.callback_dispatcher.dispatch(
            dependents,
            Some(Box::new(move || {
                if let Some(observers) = node.release_dependency_continuations() {
                    dispatch_terminal_observer_batch(node, dispatcher, observers);
                }
            })),
        );
    }

    pub(super) fn task_node(&self) -> Arc<TaskNode> {
        Arc::clone(&self.node)
    }

    pub(super) fn task_node_ref(&self) -> &TaskNode {
        &self.node
    }

    pub(super) fn descriptor(&self) -> &TaskDescriptor {
        self.node.descriptor()
    }

    pub(super) fn belongs_to_graph(&self, graph_owner: &Arc<()>) -> bool {
        self.node.belongs_to_graph(graph_owner)
    }

    #[cfg(test)]
    pub(super) fn task_node_identity(&self) -> usize {
        Arc::as_ptr(&self.node) as usize
    }

    #[cfg(test)]
    pub(super) fn dependent_count(&self) -> usize {
        self.node.lock_inner().dependents.len()
    }

    #[cfg(test)]
    pub(super) fn graph_owner_weak_count(&self) -> usize {
        self.node.graph_owner_weak_count()
    }
}

fn dispatch_terminal_observer_batch(
    node: Arc<TaskNode>,
    dispatcher: TaskCallbackDispatcher,
    observers: Vec<JobTerminalObserver>,
) {
    let callbacks = observers
        .into_iter()
        .map(|observer| {
            let node = Arc::clone(&node);
            Box::new(move || node.run_terminal_observer(observer)) as TaskCallback
        })
        .collect();
    let node_for_completion = Arc::clone(&node);
    let dispatcher_for_completion = dispatcher.clone();
    dispatcher.dispatch(
        callbacks,
        Some(Box::new(move || {
            if let Some(observers) = node_for_completion.finish_terminal_observer_batch() {
                dispatch_terminal_observer_batch(
                    node_for_completion,
                    dispatcher_for_completion,
                    observers,
                );
            }
        })),
    );
}

fn default_callback_dispatcher() -> TaskCallbackDispatcher {
    TaskCallbackDispatcher::inline()
}

impl Default for JobHandle {
    fn default() -> Self {
        Self::completed()
    }
}

impl fmt::Debug for JobHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobHandle")
            .field("is_complete", &self.is_complete())
            .field("terminal_state", &self.terminal_state())
            .finish()
    }
}

#[cfg(test)]
#[path = "job_handle/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "job_handle/tests/astra_combine_tests.rs"]
mod astra_combine_tests;
