use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use super::JobSchedulerDiagnosticsState;

type JobContinuation = Box<dyn FnOnce() + Send + 'static>;

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
            let combined_for_callback = combined.clone();
            let callback = Box::new(move || {
                if combined_for_callback.dependency_completed() {
                    combined_for_callback.mark_complete();
                }
            });
            if !handle.add_dependent(callback) && combined.dependency_completed() {
                combined.mark_complete();
            }
        }
        combined
    }

    pub fn is_complete(&self) -> bool {
        self.state
            .inner
            .lock()
            .expect("job state lock poisoned")
            .is_complete
    }

    pub fn wait(&self) {
        let started_at = Instant::now();
        let mut inner = self.state.inner.lock().expect("job state lock poisoned");
        while !inner.is_complete {
            inner = self
                .state
                .complete
                .wait(inner)
                .expect("job state lock poisoned while waiting");
        }
        if let Some(diagnostics) = &self.wait_diagnostics {
            diagnostics.record_main_thread_wait(started_at.elapsed());
        }
    }

    pub(super) fn mark_complete(&self) {
        let dependents = {
            let mut inner = self.state.inner.lock().expect("job state lock poisoned");
            if inner.is_complete {
                return;
            }
            inner.is_complete = true;
            std::mem::take(&mut inner.dependents)
        };

        self.state.complete.notify_all();
        for dependent in dependents {
            dependent();
        }
    }

    pub(super) fn add_dependent(&self, dependent: JobContinuation) -> bool {
        let mut inner = self.state.inner.lock().expect("job state lock poisoned");
        if inner.is_complete {
            false
        } else {
            inner.dependents.push(dependent);
            true
        }
    }

    pub(super) fn dependency_completed(&self) -> bool {
        let mut inner = self.state.inner.lock().expect("job state lock poisoned");
        if inner.remaining_dependencies == 0 {
            return false;
        }

        inner.remaining_dependencies -= 1;
        inner.remaining_dependencies == 0 && !inner.is_complete
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
