use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::test_render_framework::TestRenderFramework;
use super::viewport_state::ViewportState;
use crate::core::jobs::{
    test_job_system, test_job_system_with_limits, CancellationToken, EditorJob, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, JobTicket,
};
use crate::scene::viewport::RenderFramework;

#[test]
fn viewport_consumes_successful_framework_ticket_once() {
    let jobs = test_job_system();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("viewport success fixture", JobCategory::Misc),
            SuccessfulFrameworkJob,
        )
        .expect("viewport success fixture should submit");
    let mut state = ViewportState::new(None, None);
    state.jobs = Some(jobs);
    state.render_framework_task = Some(ticket);

    let resolved = poll_framework_until_terminal(&mut state)
        .expect("successful ticket should resolve a render framework");
    assert!(state.render_framework_task.is_none());
    let second = state
        .poll_or_start_render_framework()
        .expect("resolved framework should remain available")
        .expect("resolved framework should be returned");
    assert!(Arc::ptr_eq(&resolved, &second));
}

#[test]
fn viewport_maps_failed_framework_ticket_and_clears_it() {
    let jobs = test_job_system();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("viewport failure fixture", JobCategory::Misc),
            FailedFrameworkJob,
        )
        .expect("viewport failure fixture should submit");
    let mut state = ViewportState::new(None, None);
    state.jobs = Some(jobs);
    state.render_framework_task = Some(ticket);

    let error = match poll_framework_until_terminal(&mut state) {
        Ok(_) => panic!("failed ticket should become a viewport error"),
        Err(error) => error,
    };
    assert!(error
        .to_string()
        .contains("planned viewport resolve failure"));
    assert!(state.render_framework_task.is_none());
}

#[test]
fn dropping_viewport_cancels_pending_ticket_and_releases_dependents() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let (started_sender, started_receiver) = channel();
    let (release_sender, release_receiver) = sync_channel(1);
    let blocker = jobs
        .submit(
            EditorJobSpec::new("viewport cancellation blocker", JobCategory::Export),
            GateJob {
                started: started_sender,
                release: release_receiver,
            },
        )
        .expect("blocker should submit");
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("blocker should occupy the export category");

    let cancel = CancellationToken::default();
    let pending_ran = Arc::new(AtomicBool::new(false));
    let pending = jobs
        .submit(
            EditorJobSpec::new("pending viewport resolve", JobCategory::Export)
                .with_cancel(cancel.clone()),
            PendingFrameworkJob {
                ran: Arc::clone(&pending_ran),
            },
        )
        .expect("pending viewport resolve should submit");
    let dependent = jobs
        .submit(
            EditorJobSpec::new("viewport cancellation dependent", JobCategory::Misc)
                .after(pending.id()),
            SignalJob,
        )
        .expect("dependent should submit");

    let mut state = ViewportState::new(None, None);
    state.jobs = Some(jobs.clone());
    state.render_framework_cancel = Some(cancel.clone());
    state.render_framework_task = Some(pending);
    drop(state);

    assert!(cancel.is_cancelled());
    assert_eq!(take_before_deadline(&dependent), Ok("dependent released"));
    assert!(!pending_ran.load(Ordering::SeqCst));

    release_sender.send(()).expect("blocker should release");
    assert_eq!(take_before_deadline(&blocker), Ok(()));
}

fn poll_framework_until_terminal(
    state: &mut ViewportState,
) -> Result<Arc<dyn RenderFramework>, crate::scene::viewport::RenderFrameworkError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match state.poll_or_start_render_framework() {
            Ok(Some(framework)) => return Ok(framework),
            Ok(None) if Instant::now() < deadline => std::thread::yield_now(),
            Ok(None) => panic!("viewport framework ticket missed its deadline"),
            Err(error) => return Err(error),
        }
    }
}

fn take_before_deadline<T>(ticket: &JobTicket<T>) -> Result<T, JobError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = ticket.try_take() {
            return result;
        }
        assert!(Instant::now() < deadline, "job ticket missed its deadline");
        std::thread::yield_now();
    }
}

struct SuccessfulFrameworkJob;

impl EditorJob for SuccessfulFrameworkJob {
    type Output = Arc<dyn RenderFramework>;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(Arc::new(TestRenderFramework))
    }
}

struct FailedFrameworkJob;

impl EditorJob for FailedFrameworkJob {
    type Output = Arc<dyn RenderFramework>;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Err(JobError::failed(std::io::Error::other(
            "planned viewport resolve failure",
        )))
    }
}

struct PendingFrameworkJob {
    ran: Arc<AtomicBool>,
}

impl EditorJob for PendingFrameworkJob {
    type Output = Arc<dyn RenderFramework>;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.ran.store(true, Ordering::SeqCst);
        Ok(Arc::new(TestRenderFramework))
    }
}

struct GateJob {
    started: Sender<()>,
    release: Receiver<()>,
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).map_err(JobError::failed)?;
        self.release.recv().map_err(JobError::failed)
    }
}

struct SignalJob;

impl EditorJob for SignalJob {
    type Output = &'static str;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok("dependent released")
    }
}
