use std::fmt;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use super::pool::{assist_current_thread_once, TaskPoolYield};
use super::JobSchedulerDiagnosticsState;

type JobContinuation = Box<dyn FnOnce() + Send + 'static>;
type JobTerminalObserver = Box<dyn FnOnce() + Send + 'static>;
const WORKER_WAIT_IDLE_PARK: Duration = Duration::from_millis(1);

#[derive(Clone)]
pub struct JobHandle {
    state: Arc<JobState>,
    wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
}

struct JobState {
    inner: Mutex<JobStateInner>,
    complete: Condvar,
    terminal_observer_panics: AtomicUsize,
}

struct JobStateInner {
    is_complete: bool,
    dependency_continuations_published: bool,
    panic_message: Option<Arc<str>>,
    remaining_dependencies: usize,
    dependents: Vec<JobContinuation>,
    terminal_observers: Vec<JobTerminalObserver>,
}

impl JobHandle {
    pub(super) fn pending_with_dependencies(remaining_dependencies: usize) -> Self {
        Self::pending_with_wait_diagnostics(remaining_dependencies, None)
    }

    pub(super) fn pending_with_scheduler_diagnostics(
        remaining_dependencies: usize,
        wait_diagnostics: Arc<JobSchedulerDiagnosticsState>,
    ) -> Self {
        Self::pending_with_wait_diagnostics(remaining_dependencies, Some(wait_diagnostics))
    }

    fn pending_with_wait_diagnostics(
        remaining_dependencies: usize,
        wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
    ) -> Self {
        Self {
            state: Arc::new(JobState {
                inner: Mutex::new(JobStateInner {
                    is_complete: false,
                    dependency_continuations_published: false,
                    panic_message: None,
                    remaining_dependencies,
                    dependents: Vec::new(),
                    terminal_observers: Vec::new(),
                }),
                complete: Condvar::new(),
                terminal_observer_panics: AtomicUsize::new(0),
            }),
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

        let combined = Self::pending_with_wait_diagnostics(handles.len(), wait_diagnostics);
        for handle in handles {
            let handle_for_callback = handle.clone();
            let combined_for_callback = combined.clone();
            let callback = Box::new(move || {
                combined_for_callback
                    .combined_dependency_completed(handle_for_callback.panic_message());
            });
            if !handle.add_dependent(callback) {
                combined.combined_dependency_completed(handle.panic_message());
            }
        }
        combined
    }

    pub fn is_complete(&self) -> bool {
        self.state.lock_inner().is_complete
    }

    /// Runs `observer` once after this handle reaches any terminal state.
    ///
    /// Registration after dependency continuations have been published invokes the observer before
    /// this method returns. Registration during continuation publication joins the queued observers,
    /// which run outside the state lock after every continuation has been released. `wait()`
    /// synchronizes terminal state, not observer completion, so observers must stay bounded and must
    /// own any stronger completion signal their consumer requires.
    pub fn on_terminal(&self, observer: impl FnOnce() + Send + 'static) {
        let observer: JobTerminalObserver = Box::new(observer);
        let observer = {
            let mut inner = self.state.lock_inner();
            if inner.is_complete && inner.dependency_continuations_published {
                Some(observer)
            } else {
                inner.terminal_observers.push(observer);
                None
            }
        };

        if let Some(observer) = observer {
            self.state.run_terminal_observer(observer);
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
        while !inner.is_complete {
            drop(inner);
            if let Some(result) = assist_current_thread_once() {
                inner = self.state.lock_inner();
                if !inner.is_complete && result == TaskPoolYield::Idle {
                    inner = self.state.wait_inner_timeout(inner, WORKER_WAIT_IDLE_PARK);
                }
            } else {
                inner = self.state.lock_inner();
                if !inner.is_complete {
                    inner = self.state.wait_inner(inner);
                }
            }
        }
        inner.panic_message.clone()
    }

    pub(super) fn mark_complete(&self) {
        self.mark_terminal(None);
    }

    pub(super) fn mark_panicked(&self, panic_message: impl Into<Arc<str>>) {
        self.mark_terminal(Some(panic_message.into()));
    }

    pub(super) fn panic_message(&self) -> Option<Arc<str>> {
        self.state.lock_inner().panic_message.clone()
    }

    fn mark_terminal(&self, panic_message: Option<Arc<str>>) {
        let dependents = {
            let mut inner = self.state.lock_inner();
            if inner.is_complete {
                return;
            }
            inner.is_complete = true;
            inner.panic_message = panic_message;
            std::mem::take(&mut inner.dependents)
        };

        self.state.publish_terminal(dependents);
    }

    pub(super) fn add_dependent(&self, dependent: JobContinuation) -> bool {
        let mut inner = self.state.lock_inner();
        if inner.is_complete {
            false
        } else {
            inner.dependents.push(dependent);
            true
        }
    }

    pub(super) fn dependency_completed(&self) -> bool {
        let mut inner = self.state.lock_inner();
        if inner.is_complete || inner.remaining_dependencies == 0 {
            return false;
        }

        inner.remaining_dependencies -= 1;
        inner.remaining_dependencies == 0 && !inner.is_complete
    }

    fn combined_dependency_completed(&self, panic_message: Option<Arc<str>>) {
        let dependents = {
            let mut inner = self.state.lock_inner();
            if inner.is_complete || inner.remaining_dependencies == 0 {
                return;
            }
            if inner.panic_message.is_none() {
                inner.panic_message = panic_message;
            }
            inner.remaining_dependencies -= 1;
            if inner.remaining_dependencies != 0 {
                return;
            }
            inner.is_complete = true;
            std::mem::take(&mut inner.dependents)
        };

        self.state.publish_terminal(dependents);
    }
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

    fn publish_terminal(&self, dependents: Vec<JobContinuation>) {
        self.complete.notify_all();

        let mut first_continuation_panic = None;
        for dependent in dependents {
            if let Err(payload) = catch_unwind(AssertUnwindSafe(dependent)) {
                first_continuation_panic.get_or_insert(payload);
            }
        }
        let terminal_observers = {
            let mut inner = self.lock_inner();
            inner.dependency_continuations_published = true;
            std::mem::take(&mut inner.terminal_observers)
        };
        for observer in terminal_observers {
            self.run_terminal_observer(observer);
        }
        if let Some(payload) = first_continuation_panic {
            resume_unwind(payload);
        }
    }

    fn run_terminal_observer(&self, observer: JobTerminalObserver) {
        if catch_unwind(AssertUnwindSafe(observer)).is_err() {
            self.terminal_observer_panics
                .fetch_add(1, Ordering::Release);
        }
    }
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
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };
    use std::thread;
    use std::time::Duration;

    use super::JobHandle;

    #[test]
    fn job_handle_accessors_recover_poisoned_state_lock() {
        let handle = JobHandle::pending_with_dependencies(1);

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.state.inner.lock().unwrap();
            panic!("poison job handle state");
        }));

        assert!(!handle.is_complete());
        let dependent_ran = Arc::new(AtomicBool::new(false));
        let dependent_ran_for_callback = Arc::clone(&dependent_ran);
        assert!(handle.add_dependent(Box::new(move || {
            dependent_ran_for_callback.store(true, Ordering::SeqCst);
        })));
        assert!(handle.dependency_completed());
        handle.mark_complete();

        assert!(handle.is_complete());
        assert!(handle.panic_message().is_none());
        assert!(dependent_ran.load(Ordering::SeqCst));
    }

    #[test]
    fn job_handle_wait_recovers_poisoned_state_lock() {
        let handle = JobHandle::pending_with_dependencies(0);

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.state.inner.lock().unwrap();
            panic!("poison job handle wait state");
        }));

        let completer = handle.clone();
        let completion_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            completer.mark_complete();
        });

        handle.wait();
        completion_thread.join().unwrap();
        assert!(handle.is_complete());
    }

    #[test]
    fn combined_dependency_terminal_path_uses_one_state_lock() {
        let source = include_str!("job_handle.rs");
        let start = source
            .find("fn combined_dependency_completed")
            .expect("combined dependency implementation");
        let end = source[start..]
            .find("impl JobState")
            .map(|offset| start + offset)
            .expect("job state implementation");
        let implementation = &source[start..end];

        assert!(implementation.contains("inner.is_complete = true;"));
        assert!(implementation.contains("std::mem::take(&mut inner.dependents)"));
        assert!(!implementation.contains("self.panic_message()"));
        assert!(!implementation.contains("self.mark_panicked"));
        assert!(!implementation.contains("self.mark_complete"));
    }

    #[test]
    fn job_terminal_observer_runs_once_when_dependency_continuation_unwinds() {
        let handle = JobHandle::pending_with_dependencies(0);
        let sibling = JobHandle::pending_with_dependencies(0);
        let observer_runs = Arc::new(AtomicUsize::new(0));
        let observer_runs_for_callback = Arc::clone(&observer_runs);
        assert!(handle.add_dependent(Box::new(|| {
            panic!("dependency continuation failure");
        })));
        let combined = JobHandle::combine(&[handle.clone(), sibling.clone()]);
        handle.on_terminal(move || {
            observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
        });

        let terminal_result = panic::catch_unwind(AssertUnwindSafe(|| handle.mark_complete()));

        assert!(terminal_result.is_err());
        assert!(handle.is_complete());
        assert!(
            !combined.is_complete(),
            "the survivor continuation must decrement the combined barrier"
        );
        sibling.mark_complete();
        assert!(
            combined.is_complete(),
            "the continuation after the panic must preserve combined-barrier completion"
        );
        combined.wait();
        handle.wait();
        handle.mark_complete();
        assert_eq!(observer_runs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn late_terminal_observer_waits_until_dependency_continuations_finish() {
        let handle = JobHandle::pending_with_dependencies(0);
        let continuation_entered = Arc::new(std::sync::Barrier::new(2));
        let release_continuation = Arc::new(std::sync::Barrier::new(2));
        let observer_ran = Arc::new(AtomicBool::new(false));

        let continuation_entered_for_callback = Arc::clone(&continuation_entered);
        let release_continuation_for_callback = Arc::clone(&release_continuation);
        assert!(handle.add_dependent(Box::new(move || {
            continuation_entered_for_callback.wait();
            release_continuation_for_callback.wait();
        })));

        let handle_for_completion = handle.clone();
        let completion = thread::spawn(move || handle_for_completion.mark_complete());
        continuation_entered.wait();

        let observer_ran_for_callback = Arc::clone(&observer_ran);
        handle.on_terminal(move || {
            observer_ran_for_callback.store(true, Ordering::SeqCst);
        });
        assert!(
            !observer_ran.load(Ordering::SeqCst),
            "a late observer must not overtake an in-flight dependency continuation"
        );

        release_continuation.wait();
        completion.join().unwrap();
        assert!(observer_ran.load(Ordering::SeqCst));
    }
}
