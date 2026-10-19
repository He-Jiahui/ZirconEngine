use super::super::pending::PendingJob;
use super::super::pending_task::PendingTask;
use super::*;
use crate::core::jobs::{
    test_job_scheduler, test_job_system, EditorJobLimits, EditorJobProgressObserver,
    EditorJobShutdownState,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

fn enqueue(
    jobs: &EditorJobSystem,
    task: Box<dyn PendingTask>,
    cancel: Box<dyn FnOnce(JobContext) + Send + 'static>,
) -> JobId {
    let mut state = jobs.inner.lock_state();
    let id = state.allocate_id();
    let spec = EditorJobSpec::new("owned shutdown fixture", JobCategory::Misc);
    state.register(id);
    jobs.inner.progress.register(id, &spec);
    state.enqueue_pending(PendingJob::new(id, spec, task, cancel, Instant::now()));
    id
}

fn wait_until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "owned shutdown did not become terminal"
        );
        std::thread::yield_now();
    }
}

#[test]
fn cleanup_installs_real_handle_before_callback_and_polls_until_terminal() {
    let jobs = test_job_system();
    let callback_jobs = jobs.clone();
    let caller = std::thread::current().id();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = Arc::clone(&calls);
    enqueue(
        &jobs,
        Box::new(|_: JobContext| {}),
        Box::new(move |_| {
            callback_calls.fetch_add(1, Ordering::AcqRel);
            let installed = callback_jobs
                .inner
                .lock_state()
                .shutdown_cleanup()
                .is_some();
            started_tx
                .send((std::thread::current().id(), installed))
                .unwrap();
            release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }),
    );

    jobs.request_shutdown();
    let (callback_thread, installed) = started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_ne!(callback_thread, caller);
    assert!(installed);
    let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
    assert!(!cleanup.is_complete());
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::Pending
    );
    jobs.request_shutdown();
    release_tx.send(()).unwrap();
    wait_until(|| jobs.shutdown_progress().state() == EditorJobShutdownState::Quiescent);
    assert_eq!(cleanup.terminal_state(), Some(TaskState::Completed));
    assert_eq!(calls.load(Ordering::Acquire), 1);
}

#[test]
fn empty_progress_stays_pending_while_real_completion_barrier_is_owned() {
    let jobs = test_job_system();
    let barrier = CompletionBarrierGuard::new(Arc::clone(&jobs.inner));
    jobs.request_shutdown();
    let pending = jobs.shutdown_progress();
    assert!(pending.unfinished_jobs().is_empty());
    assert!(pending.completion_callbacks_pending());
    assert_eq!(pending.state(), EditorJobShutdownState::Pending);
    drop(barrier);
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::Quiescent
    );
}

#[test]
fn cleanup_runtime_failure_is_sticky_and_never_quiescent() {
    let jobs = test_job_system();
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = Arc::clone(&calls);
    enqueue(
        &jobs,
        Box::new(|_: JobContext| {}),
        Box::new(move |_| {
            callback_calls.fetch_add(1, Ordering::AcqRel);
            panic!("real cleanup callback failure");
        }),
    );
    jobs.request_shutdown();
    let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
    wait_until(|| cleanup.is_complete());
    assert_eq!(cleanup.terminal_state(), Some(TaskState::Failed));
    let failed = jobs.shutdown_progress();
    assert_eq!(failed.state(), EditorJobShutdownState::Failed);
    assert!(matches!(
        failed.cleanup_failure_kind(),
        Some(EditorJobShutdownFailure::CleanupTaskFailed)
    ));
    jobs.request_shutdown();
    assert_eq!(calls.load(Ordering::Acquire), 1);
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::Failed
    );
}

#[derive(Default)]
struct FinishedObserver(AtomicUsize);

impl EditorJobProgressObserver for FinishedObserver {
    fn job_admitted(&self, _: JobId, _: &EditorJobProgressSource) {}
    fn job_finished(&self, _: JobId, _: &EditorJobProgressSource) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
    fn jobs_resynchronized(&self, _: &EditorJobProgressSource) {}
}

#[test]
fn prior_real_task_failure_does_not_skip_remaining_cancel_or_finished_observer() {
    let observer = Arc::new(FinishedObserver::default());
    let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
        test_job_scheduler(),
        Default::default(),
        EditorJobLimits::default(),
        observer.clone(),
    );
    let failed_id = enqueue(
        &jobs,
        Box::new(|_: JobContext| {
            panic!("raw runtime task failure outside EditorJob result wrapper");
        }),
        Box::new(|_| {}),
    );
    jobs.inner.promote();
    wait_until(|| {
        matches!(
            jobs.shutdown_progress().cleanup_failure_kind(),
            Some(EditorJobShutdownFailure::JobTaskFailed { job, .. }) if *job == failed_id
        )
    });
    let finished_before = observer.0.load(Ordering::Acquire);
    let calls = Arc::new(AtomicUsize::new(0));
    let cancel_calls = Arc::clone(&calls);
    let pending_id = enqueue(
        &jobs,
        Box::new(|_: JobContext| {}),
        Box::new(move |_| {
            cancel_calls.fetch_add(1, Ordering::AcqRel);
        }),
    );
    jobs.request_shutdown();
    let cleanup = jobs.inner.lock_state().shutdown_cleanup().unwrap().clone();
    wait_until(|| cleanup.is_complete());
    assert_eq!(cleanup.terminal_state(), Some(TaskState::Completed));
    assert_eq!(calls.load(Ordering::Acquire), 1);
    assert!(jobs.is_terminal_record(pending_id));
    assert_eq!(observer.0.load(Ordering::Acquire), finished_before + 1);
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::Failed
    );
    jobs.request_shutdown();
    assert_eq!(calls.load(Ordering::Acquire), 1);
}
