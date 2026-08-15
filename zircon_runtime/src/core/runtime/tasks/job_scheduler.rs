//! Runtime scheduler facade for compute work submitted through the core task pools.

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::diagnostics::DiagnosticStore;

use super::{
    JobHandle, JobSchedulerDiagnosticsState, JobSchedulerReport, TaskPool, TaskPoolDescriptor,
    TaskPools,
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

    /// Returns a scheduler backed by the process-wide I/O pool.
    pub fn process_io() -> Self {
        Self::from_pool(TaskPools::process_default().io().clone())
    }

    /// Enables bounded lifecycle diagnostics before work is submitted to this scheduler.
    pub fn with_diagnostics(self) -> Self {
        self.diagnostics.enable();
        self
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        let diagnostics = Arc::clone(&self.diagnostics);
        let enqueued_at = diagnostics.record_scheduled_and_enqueued();
        self.pool.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            run_detached_task(diagnostics, execution_started_at, task);
        });
    }

    pub fn schedule(&self, task: impl FnOnce() + Send + 'static) -> JobHandle {
        let handle =
            JobHandle::pending_with_scheduler_diagnostics(0, Arc::clone(&self.diagnostics));
        let handle_for_task = handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        let enqueued_at = diagnostics.record_scheduled_and_enqueued();
        self.pool.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            complete_scheduled_task(
                handle_for_task,
                diagnostics,
                execution_started_at,
                move || {
                    task();
                },
            );
        });
        handle
    }

    pub fn schedule_after(
        &self,
        dependencies: &[JobHandle],
        task: impl FnOnce() + Send + 'static,
    ) -> JobHandle {
        if dependencies.is_empty() {
            return self.schedule(task);
        }

        let diagnostics_tracked = self.diagnostics.record_scheduled();
        let handle = JobHandle::pending_with_scheduler_diagnostics(
            dependencies.len(),
            Arc::clone(&self.diagnostics),
        );
        let pending = Arc::new(PendingScheduledJob {
            pool: self.pool.clone(),
            handle: handle.clone(),
            diagnostics: Arc::clone(&self.diagnostics),
            created_at: diagnostics_tracked.then(Instant::now),
            diagnostics_tracked,
            dependency_count: dependencies.len(),
            task: Mutex::new(Some(Box::new(task))),
        });

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
    created_at: Option<Instant>,
    diagnostics_tracked: bool,
    dependency_count: usize,
    task: Mutex<Option<ScheduledJob>>,
}

impl PendingScheduledJob {
    fn try_launch(&self) {
        let Some(task) = self.lock_task().take() else {
            return;
        };
        if self.dependency_count > 0 {
            self.diagnostics.record_dependency_wait(self.created_at);
        }
        let handle = self.handle.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        let enqueued_at = diagnostics.record_enqueued(self.diagnostics_tracked);
        self.pool.spawn(move || {
            let tracked = diagnostics.record_started(enqueued_at);
            let execution_started_at = diagnostics.execution_started_at(tracked);
            complete_scheduled_task(handle, diagnostics, execution_started_at, move || {
                task();
            });
        });
    }

    fn record_terminal_without_launch(&self) {
        let task_was_pending = {
            let mut task = self.lock_task();
            task.take().is_some()
        };
        if task_was_pending {
            if self.dependency_count > 0 {
                self.diagnostics.record_dependency_wait(self.created_at);
            }
            self.diagnostics.record_cancelled(self.diagnostics_tracked);
        }
    }

    fn lock_task(&self) -> MutexGuard<'_, Option<ScheduledJob>> {
        self.task
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct DetachedTaskCompletion {
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    execution_started_at: Option<Instant>,
}

impl DetachedTaskCompletion {
    fn new(
        diagnostics: Arc<JobSchedulerDiagnosticsState>,
        execution_started_at: Option<Instant>,
    ) -> Self {
        Self {
            diagnostics,
            execution_started_at,
        }
    }
}

impl Drop for DetachedTaskCompletion {
    fn drop(&mut self) {
        self.diagnostics
            .record_active_terminal(std::thread::panicking(), self.execution_started_at);
    }
}

fn run_detached_task(
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    execution_started_at: Option<Instant>,
    task: impl FnOnce(),
) {
    let _completion = DetachedTaskCompletion::new(diagnostics, execution_started_at);
    task();
}

fn complete_scheduled_task(
    handle: JobHandle,
    diagnostics: Arc<JobSchedulerDiagnosticsState>,
    execution_started_at: Option<Instant>,
    task: impl FnOnce(),
) {
    let result = catch_unwind(AssertUnwindSafe(task));
    diagnostics.record_active_terminal(result.is_err(), execution_started_at);
    match result {
        Ok(()) => handle.mark_complete(),
        Err(payload) => handle.mark_panicked(panic_payload_message(payload)),
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::process::Command;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use std::time::{Duration, Instant};

    use super::{
        run_detached_task, JobHandle, JobScheduler, JobSchedulerDiagnosticsState,
        PendingScheduledJob, TaskPool, TaskPoolDescriptor, TaskPools,
    };

    #[test]
    fn process_io_uses_the_shared_runtime_io_pool() {
        assert_eq!(
            JobScheduler::process_io().parallelism(),
            TaskPools::process_default().io().parallelism()
        );
    }

    #[test]
    fn scheduled_task_records_one_execution_sample_after_completion() {
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(1),
        ))
        .with_diagnostics();

        scheduler.schedule(|| {}).wait();

        let report = scheduler.diagnostic_report();
        assert_eq!(report.scheduled, 1);
        assert_eq!(report.completed, 1);
        assert_eq!(report.execution_samples, 1);
        assert!(report.execution_ms >= 0.0);
    }

    #[test]
    fn detached_spawn_counts_panicked_tasks_as_completed() {
        const CHILD_ENV: &str = "ZIRCON_DETACHED_PANIC_DIAGNOSTICS_CHILD";
        const CHILD_STARTED: &str = "zircon detached panic child started";
        const CHILD_SURVIVED_EXIT_CODE: i32 = 91;

        if std::env::var_os(CHILD_ENV).is_some() {
            eprintln!("{CHILD_STARTED}");
            let scheduler = JobScheduler::from_pool(TaskPool::new(
                TaskPoolDescriptor::compute().with_worker_threads(1),
            ));
            scheduler.spawn(|| panic!("detached task failure"));
            std::thread::sleep(Duration::from_secs(2));
            std::process::exit(CHILD_SURVIVED_EXIT_CODE);
        }

        let diagnostics = Arc::new(JobSchedulerDiagnosticsState::default());
        diagnostics.enable();
        let enqueued_at = diagnostics.record_scheduled_and_enqueued();
        let tracked = diagnostics.record_started(enqueued_at);
        let execution_started_at = diagnostics.execution_started_at(tracked);
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            run_detached_task(Arc::clone(&diagnostics), execution_started_at, || {
                panic!("detached task failure")
            });
        }));
        assert!(result.is_err());
        let report = diagnostics.report();
        assert_eq!(report.scheduled, 1);
        assert_eq!(report.completed, 1);
        assert_eq!(report.panicked, 1);
        assert_eq!(report.queued, 0);
        assert_eq!(report.active, 0);
        assert_eq!(report.execution_samples, 1);

        let test_executable = std::env::current_exe().expect("current lib-test executable");
        let listed = Command::new(&test_executable)
            .arg("--list")
            .output()
            .expect("list current lib-test names");
        assert!(
            listed.status.success(),
            "current lib-test list must succeed"
        );
        let test_suffix = "::detached_spawn_counts_panicked_tasks_as_completed";
        let listed_stdout = String::from_utf8_lossy(&listed.stdout);
        let test_name = listed_stdout
            .lines()
            .filter_map(|line| line.strip_suffix(": test"))
            .find(|name| name.ends_with(test_suffix))
            .unwrap_or_else(|| panic!("lib-test list should contain `{test_suffix}`"))
            .to_owned();

        let output = Command::new(test_executable)
            .args(["--exact", test_name.as_str(), "--nocapture"])
            .env(CHILD_ENV, "1")
            .output()
            .expect("launch isolated detached-panic diagnostic test");

        assert!(
            !output.status.success(),
            "real Rayon detached panic must retain its process-terminating default"
        );
        assert_ne!(
            output.status.code(),
            Some(CHILD_SURVIVED_EXIT_CODE),
            "detached task did not reach Rayon's panic termination path"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(CHILD_STARTED));
        assert!(
            stderr.contains("detached task failure"),
            "child stderr should prove the selected detached task actually panicked: {stderr}"
        );
    }

    #[test]
    fn pending_scheduled_job_recovers_poisoned_task_lock() {
        let diagnostics = Arc::new(JobSchedulerDiagnosticsState::default());
        let handle = JobHandle::pending_with_scheduler_diagnostics(0, Arc::clone(&diagnostics));
        let task_ran = Arc::new(AtomicBool::new(false));
        let task_ran_for_job = Arc::clone(&task_ran);
        let pending = PendingScheduledJob {
            pool: TaskPool::new(TaskPoolDescriptor::compute()),
            handle: handle.clone(),
            diagnostics,
            created_at: None,
            diagnostics_tracked: false,
            dependency_count: 0,
            task: Mutex::new(Some(Box::new(move || {
                task_ran_for_job.store(true, Ordering::SeqCst);
            }))),
        };

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = pending.task.lock().unwrap();
            panic!("poison pending scheduled job task");
        }));

        pending.try_launch();
        handle.wait();
        assert!(task_ran.load(Ordering::SeqCst));
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
