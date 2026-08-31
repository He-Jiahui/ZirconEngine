use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use super::callback_dispatcher::{TaskCallback, TaskCallbackDispatcher};
use super::pool::{assist_current_thread_once, TaskPoolYield};
use super::{JobSchedulerDiagnosticsState, TaskId, TaskState, TaskStatus};

type JobContinuation = TaskCallback;
type JobTerminalObserver = TaskCallback;
const WORKER_WAIT_IDLE_PARK: Duration = Duration::from_millis(1);

#[derive(Clone)]
pub struct JobHandle {
    state: Arc<JobState>,
    callback_dispatcher: TaskCallbackDispatcher,
    wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
}

struct JobState {
    inner: Mutex<JobStateInner>,
    complete: Condvar,
    terminal_observer_panics: AtomicUsize,
}

struct JobStateInner {
    lifecycle: TaskState,
    is_cancelled: bool,
    dependency_continuations_published: bool,
    terminal_observer_delivery_active: bool,
    panic_message: Option<Arc<str>>,
    remaining_dependencies: usize,
    dependents: Vec<JobContinuation>,
    terminal_observers: Vec<JobTerminalObserver>,
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
        Self {
            state: Arc::new(JobState {
                inner: Mutex::new(JobStateInner {
                    lifecycle: TaskState::Pending,
                    is_cancelled: false,
                    dependency_continuations_published: false,
                    terminal_observer_delivery_active: false,
                    panic_message: None,
                    remaining_dependencies,
                    dependents: Vec::new(),
                    terminal_observers: Vec::new(),
                }),
                complete: Condvar::new(),
                terminal_observer_panics: AtomicUsize::new(0),
            }),
            callback_dispatcher,
            wait_diagnostics,
        }
    }

    pub fn completed() -> Self {
        let handle = Self::pending_with_dependencies(0);
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
            let handle_for_callback = handle.clone();
            let combined_for_callback = combined.clone();
            let callback = Box::new(move || {
                combined_for_callback.combined_dependency_completed(
                    handle_for_callback.panic_message(),
                    handle_for_callback.is_cancelled(),
                );
            });
            if !handle.add_dependent(callback) {
                combined
                    .combined_dependency_completed(handle.panic_message(), handle.is_cancelled());
            }
        }
        combined
    }

    pub fn is_complete(&self) -> bool {
        self.state.lock_inner().lifecycle.is_terminal()
    }

    pub fn is_cancelled(&self) -> bool {
        let inner = self.state.lock_inner();
        inner.lifecycle == TaskState::Cancelled
    }

    pub fn terminal_state(&self) -> Option<TaskState> {
        let inner = self.state.lock_inner();
        inner.lifecycle.is_terminal().then_some(inner.lifecycle)
    }

    pub(super) fn task_status(&self, id: TaskId) -> TaskStatus {
        let inner = self.state.lock_inner();
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
            let mut inner = self.state.lock_inner();
            inner.terminal_observers.push(observer);
            if inner.lifecycle.is_terminal() && inner.dependency_continuations_published {
                JobState::take_terminal_observer_batch(&mut inner)
            } else {
                None
            }
        };

        if let Some(observers) = observers {
            dispatch_terminal_observer_batch(
                Arc::clone(&self.state),
                self.callback_dispatcher.clone(),
                observers,
            );
        }
    }

    /// Returns the number of terminal observers whose panics were contained for this handle.
    pub fn terminal_observer_panic_count(&self) -> usize {
        self.state.terminal_observer_panics.load(Ordering::Acquire)
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
        let mut inner = self.state.lock_inner();
        while !inner.lifecycle.is_terminal() {
            drop(inner);
            if let Some(result) = assist_current_thread_once() {
                inner = self.state.lock_inner();
                if !inner.lifecycle.is_terminal() && result == TaskPoolYield::Idle {
                    inner = self.state.wait_inner_timeout(inner, WORKER_WAIT_IDLE_PARK);
                }
            } else {
                inner = self.state.lock_inner();
                if !inner.lifecycle.is_terminal() {
                    inner = self.state.wait_inner(inner);
                }
            }
        }
        inner.panic_message.clone()
    }

    pub(super) fn mark_complete(&self) {
        self.mark_terminal(None, false);
    }

    pub(super) fn mark_running(&self) {
        let mut inner = self.state.lock_inner();
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
        self.state.lock_inner().panic_message.clone()
    }

    fn mark_terminal(&self, panic_message: Option<Arc<str>>, is_cancelled: bool) {
        let dependents = {
            let mut inner = self.state.lock_inner();
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
        let mut inner = self.state.lock_inner();
        if inner.lifecycle.is_terminal() {
            false
        } else {
            inner.dependents.push(dependent);
            true
        }
    }

    pub(super) fn dependency_completed(&self) -> bool {
        let mut inner = self.state.lock_inner();
        if inner.lifecycle.is_terminal() || inner.remaining_dependencies == 0 {
            return false;
        }

        inner.remaining_dependencies -= 1;
        inner.remaining_dependencies == 0 && !inner.lifecycle.is_terminal()
    }

    fn combined_dependency_completed(&self, panic_message: Option<Arc<str>>, is_cancelled: bool) {
        let dependents = {
            let mut inner = self.state.lock_inner();
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
        self.state.complete.notify_all();
        let state = Arc::clone(&self.state);
        let dispatcher = self.callback_dispatcher.clone();
        self.callback_dispatcher.dispatch(
            dependents,
            Some(Box::new(move || {
                if let Some(observers) = state.release_dependency_continuations() {
                    dispatch_terminal_observer_batch(state, dispatcher, observers);
                }
            })),
        );
    }
}

fn dispatch_terminal_observer_batch(
    state: Arc<JobState>,
    dispatcher: TaskCallbackDispatcher,
    observers: Vec<JobTerminalObserver>,
) {
    let callbacks = observers
        .into_iter()
        .map(|observer| {
            let state = Arc::clone(&state);
            Box::new(move || state.run_terminal_observer(observer)) as TaskCallback
        })
        .collect();
    let state_for_completion = Arc::clone(&state);
    let dispatcher_for_completion = dispatcher.clone();
    dispatcher.dispatch(
        callbacks,
        Some(Box::new(move || {
            if let Some(observers) = state_for_completion.finish_terminal_observer_batch() {
                dispatch_terminal_observer_batch(
                    state_for_completion,
                    dispatcher_for_completion,
                    observers,
                );
            }
        })),
    );
}

impl JobState {
    fn lock_inner(&self) -> MutexGuard<'_, JobStateInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn wait_inner<'a>(
        &self,
        inner: MutexGuard<'a, JobStateInner>,
    ) -> MutexGuard<'a, JobStateInner> {
        self.complete
            .wait(inner)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn wait_inner_timeout<'a>(
        &self,
        inner: MutexGuard<'a, JobStateInner>,
        timeout: Duration,
    ) -> MutexGuard<'a, JobStateInner> {
        self.complete
            .wait_timeout(inner, timeout)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .0
    }

    fn release_dependency_continuations(&self) -> Option<Vec<JobTerminalObserver>> {
        let mut inner = self.lock_inner();
        inner.dependency_continuations_published = true;
        Self::take_terminal_observer_batch(&mut inner)
    }

    fn finish_terminal_observer_batch(&self) -> Option<Vec<JobTerminalObserver>> {
        let mut inner = self.lock_inner();
        inner.terminal_observer_delivery_active = false;
        Self::take_terminal_observer_batch(&mut inner)
    }

    fn take_terminal_observer_batch(inner: &mut JobStateInner) -> Option<Vec<JobTerminalObserver>> {
        if inner.terminal_observer_delivery_active || inner.terminal_observers.is_empty() {
            None
        } else {
            inner.terminal_observer_delivery_active = true;
            Some(std::mem::take(&mut inner.terminal_observers))
        }
    }

    fn run_terminal_observer(&self, observer: JobTerminalObserver) {
        if catch_unwind(AssertUnwindSafe(observer)).is_err() {
            self.terminal_observer_panics
                .fetch_add(1, Ordering::Release);
        }
    }
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
#[path = "job_handle/tests.rs"]
mod tests;
