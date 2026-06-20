//! Runtime scheduler facade for compute work submitted through the core task pools.

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::core::diagnostics::DiagnosticStore;

use super::{
    JobHandle, JobSchedulerDiagnosticsState, JobSchedulerReport, TaskPool, TaskPoolDescriptor,
};

#[derive(Clone)]
pub struct JobScheduler {
    pool: TaskPool,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
}

impl fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobScheduler")
            .field("parallelism", &self.parallelism())
            .finish()
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::from_pool(TaskPool::new(TaskPoolDescriptor::compute()))
    }
}

impl JobScheduler {
    pub(crate) fn from_pool(pool: TaskPool) -> Self {
        Self {
            pool,
            diagnostics: Arc::default(),
        }
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.diagnostics.record_scheduled();
        let diagnostics = Arc::clone(&self.diagnostics);
        self.pool.spawn(move || {
            task();
            diagnostics.record_completed();
        });
    }

    pub fn schedule(&self, task: impl FnOnce() + Send + 'static) -> JobHandle {
        self.diagnostics.record_scheduled();
        let handle =
            JobHandle::pending_with_scheduler_diagnostics(0, Arc::clone(&self.diagnostics));
        let handle_for_task = handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        self.pool.spawn(move || {
            complete_scheduled_task(handle_for_task, diagnostics, move || {
                task();
            });
        });
        handle
    }

    pub fn schedule_after(
        &self,
        dependencies: &[JobHandle],
        task: impl FnOnce() + Send + 'static,
    ) -> JobHandle {
        self.diagnostics.record_scheduled();
        let handle = JobHandle::pending_with_scheduler_diagnostics(
            dependencies.len(),
            Arc::clone(&self.diagnostics),
        );
        let pending = Arc::new(PendingScheduledJob {
            pool: self.pool.clone(),
            handle: handle.clone(),
            diagnostics: Arc::clone(&self.diagnostics),
            created_at: Instant::now(),
            dependency_count: dependencies.len(),
            task: Mutex::new(Some(Box::new(task))),
        });

        if dependencies.is_empty() {
            pending.try_launch();
            return handle;
        }

        for dependency in dependencies {
            let dependency_for_callback = dependency.clone();
            let handle_for_callback = handle.clone();
            let pending_for_callback = Arc::clone(&pending);
            let callback = Box::new(move || {
                if let Some(panic_message) = dependency_for_callback.panic_message() {
                    pending_for_callback.record_terminal_without_launch();
                    handle_for_callback.mark_panicked(panic_message);
                    return;
                }
                if handle_for_callback.dependency_completed() {
                    pending_for_callback.try_launch();
                }
            });
            if !dependency.add_dependent(callback) {
                if let Some(panic_message) = dependency.panic_message() {
                    pending.record_terminal_without_launch();
                    handle.mark_panicked(panic_message);
                } else if handle.dependency_completed() {
                    pending.try_launch();
                }
            }
        }

        handle
    }

    pub fn wait_all(&self, handles: &[JobHandle]) {
        JobHandle::combine_with_scheduler_diagnostics(handles, Arc::clone(&self.diagnostics))
            .wait();
    }

    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.pool.install(task)
    }

    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.pool.join(task_a, task_b)
    }

    pub fn parallelism(&self) -> usize {
        self.pool.parallelism()
    }

    pub fn diagnostic_report(&self) -> JobSchedulerReport {
        self.diagnostics.report()
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.diagnostics.record_diagnostics(store, frame_index);
    }
}

type ScheduledJob = Box<dyn FnOnce() + Send + 'static>;

struct PendingScheduledJob {
    pool: TaskPool,
    handle: JobHandle,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    created_at: Instant,
    dependency_count: usize,
    task: Mutex<Option<ScheduledJob>>,
}

impl PendingScheduledJob {
    fn try_launch(&self) {
        let Some(task) = self
            .task
            .lock()
            .expect("pending job task lock poisoned")
            .take()
        else {
            return;
        };
        if self.dependency_count > 0 {
            self.diagnostics
                .record_dependency_wait(self.created_at.elapsed());
        }
        let handle = self.handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        self.pool.spawn(move || {
            complete_scheduled_task(handle, diagnostics, move || {
                task();
            });
        });
    }

    fn record_terminal_without_launch(&self) {
        let task_was_pending = {
            let mut task = self.task.lock().expect("pending job task lock poisoned");
            task.take().is_some()
        };
        if task_was_pending {
            if self.dependency_count > 0 {
                self.diagnostics
                    .record_dependency_wait(self.created_at.elapsed());
            }
            self.diagnostics.record_completed();
        }
    }
}

fn complete_scheduled_task(
    handle: JobHandle,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    task: impl FnOnce(),
) {
    let result = catch_unwind(AssertUnwindSafe(task));
    diagnostics.record_completed();
    match result {
        Ok(()) => handle.mark_complete(),
        Err(payload) => handle.mark_panicked(panic_payload_message(payload)),
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
