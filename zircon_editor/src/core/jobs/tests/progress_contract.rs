use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use super::super::{
    test_job_system, test_job_system_with_limits, CancellationToken, EditorJob, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, JobTicket,
};

const TEST_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn pending_jobs_are_visible_and_cloned_progress_sources_share_sorted_snapshots() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let progress = jobs.progress();
    let cloned_progress = progress.clone();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let running = jobs
        .submit(
            EditorJobSpec::new("running", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv_timeout(TEST_TIMEOUT).unwrap();
    let pending = jobs
        .submit(EditorJobSpec::new("pending", JobCategory::Export), ValueJob)
        .unwrap();

    let snapshot = cloned_progress.snapshot();
    assert_eq!(
        snapshot.iter().map(|job| job.id()).collect::<Vec<_>>(),
        vec![running.id(), pending.id()]
    );
    assert_eq!(snapshot[1].label(), "pending");
    assert_eq!(snapshot[1].category(), JobCategory::Export);
    assert_eq!(snapshot[1].progress(), None);
    assert!(snapshot[1].cancellable());

    release_sender.send(()).unwrap();
    take_before_deadline(&running, TEST_TIMEOUT).unwrap();
    take_before_deadline(&pending, TEST_TIMEOUT).unwrap();
}

#[test]
fn reported_progress_updates_the_unique_active_job_snapshot() {
    let jobs = test_job_system();
    let progress = jobs.progress();
    let (reported_sender, reported_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("index", JobCategory::Index),
            ReportingGateJob {
                reported: reported_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    reported_receiver.recv_timeout(TEST_TIMEOUT).unwrap();

    let snapshot = progress.snapshot();
    let active = snapshot.iter().find(|job| job.id() == ticket.id()).unwrap();
    let reported = active.progress().unwrap();
    assert_eq!(reported.completed(), 4);
    assert_eq!(reported.total(), 10);
    assert_eq!(reported.message(), "indexing assets");

    release_sender.send(()).unwrap();
    take_before_deadline(&ticket, TEST_TIMEOUT).unwrap();
}

#[test]
fn completed_failed_and_cancelled_jobs_leave_the_progress_source() {
    let jobs = test_job_system();
    let progress = jobs.progress();

    let completed = jobs
        .submit(EditorJobSpec::new("completed", JobCategory::Misc), ValueJob)
        .unwrap();
    take_before_deadline(&completed, TEST_TIMEOUT).unwrap();
    let failed = jobs
        .submit(EditorJobSpec::new("failed", JobCategory::Compile), PanicJob)
        .unwrap();
    take_before_deadline(&failed, TEST_TIMEOUT).unwrap_err();
    let cancel = CancellationToken::default();
    cancel.cancel();
    let cancelled = jobs
        .submit(
            EditorJobSpec::new("cancelled", JobCategory::Thumbnail).with_cancel(cancel),
            ValueJob,
        )
        .unwrap();
    take_before_deadline(&cancelled, TEST_TIMEOUT).unwrap_err();

    wait_until(TEST_TIMEOUT, || progress.snapshot().is_empty());
}

#[test]
fn progress_store_is_the_only_active_job_lifecycle_owner() {
    let scheduler_state = include_str!("../system/state.rs");
    assert!(!scheduler_state.contains("active_jobs"));
    assert!(!scheduler_state.contains("struct ActiveJob"));
}

#[test]
fn cancel_requests_reach_running_tokens_and_terminal_events_remove_progress() {
    let jobs = test_job_system();
    let progress = jobs.progress();
    let (started_sender, started_receiver) = mpsc::channel();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("running-cancel", JobCategory::Compile),
            CooperativeJob {
                started: started_sender,
            },
        )
        .unwrap();
    started_receiver.recv_timeout(TEST_TIMEOUT).unwrap();

    assert!(jobs.cancel(ticket.id()));
    assert_eq!(
        take_before_deadline(&ticket, TEST_TIMEOUT),
        Err(JobError::Cancelled)
    );
    wait_until(TEST_TIMEOUT, || progress.snapshot().is_empty());
    assert!(!jobs.cancel(super::super::JobId::new(u64::MAX)));
}

#[test]
fn shutdown_removes_pending_entries_and_reports_only_non_terminal_canonical_entries() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let progress = jobs.progress();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let running = jobs
        .submit(
            EditorJobSpec::new("shutdown-running", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv_timeout(TEST_TIMEOUT).unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("shutdown-pending", JobCategory::Export),
            ValueJob,
        )
        .unwrap();

    let unfinished = jobs.shutdown(Instant::now());
    assert_eq!(pending.try_take(), Some(Err(JobError::Cancelled)));
    assert_eq!(unfinished.len(), 1);
    assert_eq!(unfinished[0].id(), running.id());
    assert_eq!(
        progress
            .snapshot()
            .iter()
            .map(|job| job.id())
            .collect::<Vec<_>>(),
        vec![running.id()]
    );

    release_sender.send(()).unwrap();
    take_before_deadline(&running, TEST_TIMEOUT).unwrap();
    wait_until(TEST_TIMEOUT, || progress.snapshot().is_empty());
}

struct ValueJob;

impl EditorJob for ValueJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(())
    }
}

struct PanicJob;

impl EditorJob for PanicJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        panic!("planned progress failure")
    }
}

struct GateJob {
    started: Sender<()>,
    release: Receiver<()>,
}

impl GateJob {
    fn new(started: Sender<()>, release: Receiver<()>) -> Self {
        Self { started, release }
    }
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release
            .recv_timeout(TEST_TIMEOUT)
            .expect("gate release should be sent before the test deadline");
        Ok(())
    }
}

struct ReportingGateJob {
    reported: Sender<()>,
    release: Receiver<()>,
}

impl EditorJob for ReportingGateJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.report_progress(4, 10, "indexing assets");
        self.reported.send(()).unwrap();
        self.release
            .recv_timeout(TEST_TIMEOUT)
            .expect("progress gate release should be sent before the test deadline");
        Ok(())
    }
}

struct CooperativeJob {
    started: Sender<()>,
}

impl EditorJob for CooperativeJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        loop {
            context.check_cancelled()?;
            thread::yield_now();
        }
    }
}

fn wait_until(timeout: Duration, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + timeout;
    while !condition() {
        assert!(Instant::now() < deadline, "condition missed its deadline");
        thread::yield_now();
    }
}

fn take_before_deadline<T>(ticket: &JobTicket<T>, timeout: Duration) -> Result<T, JobError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(result) = ticket.try_take() {
            return result;
        }
        assert!(Instant::now() < deadline, "job ticket missed its deadline");
        thread::yield_now();
    }
}
