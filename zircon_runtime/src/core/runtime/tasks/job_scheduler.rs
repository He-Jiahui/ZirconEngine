//! Runtime scheduler facade for compute work submitted through the core task pools.

use std::fmt;
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
            task();
            diagnostics.record_completed();
            handle_for_task.mark_complete();
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
            let handle_for_callback = handle.clone();
            let pending_for_callback = Arc::clone(&pending);
            let callback = Box::new(move || {
                if handle_for_callback.dependency_completed() {
                    pending_for_callback.try_launch();
                }
            });
            if !dependency.add_dependent(callback) && handle.dependency_completed() {
                pending.try_launch();
            }
        }

        handle
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
            task();
            diagnostics.record_completed();
            handle.mark_complete();
        });
    }
}
