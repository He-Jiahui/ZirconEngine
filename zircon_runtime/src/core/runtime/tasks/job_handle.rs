use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::pool::{assist_current_thread_once, TaskPoolYield};
use super::JobSchedulerDiagnosticsState;

type JobContinuation = Box<dyn FnOnce() + Send + 'static>;
const WORKER_WAIT_IDLE_PARK: Duration = Duration::from_millis(1);

#[derive(Clone)]
pub struct JobHandle {
    state: Arc<JobState>,
    wait_diagnostics: Option<Arc<JobSchedulerDiagnosticsState>>,
}

struct JobState {
    inner: Mutex<JobStateInner>,
    complete: Condvar,
}

struct JobStateInner {
    is_complete: bool,
    panic_message: Option<Arc<str>>,
    remaining_dependencies: usize,
    dependents: Vec<JobContinuation>,
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
                    panic_message: None,
                    remaining_dependencies,
                    dependents: Vec::new(),
                }),
                complete: Condvar::new(),
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
                if let Some(panic_message) = handle_for_callback.panic_message() {
                    combined_for_callback.mark_panicked(panic_message);
                    return;
                }
                if combined_for_callback.dependency_completed() {
                    combined_for_callback.mark_complete();
                }
            });
            if !handle.add_dependent(callback) {
                if let Some(panic_message) = handle.panic_message() {
                    combined.mark_panicked(panic_message);
                } else if combined.dependency_completed() {
                    combined.mark_complete();
                }
            }
        }
        combined
    }

    pub fn is_complete(&self) -> bool {
        self.state.lock_inner().is_complete
    }

    pub fn wait(&self) {
        let started_at = Instant::now();
        let panic_message = self.wait_for_terminal();
        if let Some(diagnostics) = &self.wait_diagnostics {
            diagnostics.record_main_thread_wait(started_at.elapsed());
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

        self.state.complete.notify_all();
        for dependent in dependents {
            dependent();
        }
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
        atomic::{AtomicBool, Ordering},
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
}
