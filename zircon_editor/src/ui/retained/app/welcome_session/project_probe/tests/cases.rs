use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

use super::job::{WelcomeProjectProbeJob, WelcomeProjectProbeResult};
use super::projection::{merge_probe_diagnostic, project_probe_projection};
use super::state::{
    WelcomeProjectProbeState, WELCOME_PROJECT_PROBE_DEBOUNCE,
    WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY,
};
use crate::core::jobs::{
    test_job_system, test_job_system_with_limits, EditorJob, EditorJobAdmissionLimits,
    EditorJobLimits, EditorJobSpec, JobCategory, JobContext, JobError, JobSubmitError,
};
use crate::core::project::ProjectAuthorityError;
use zircon_runtime_interface::project::ProjectNameError;

#[test]
fn missing_project_is_an_expected_disabled_open_state() {
    let (can_open, diagnostic) =
        project_probe_projection(Err(ProjectAuthorityError::ProjectMissing {
            path: PathBuf::from("missing-project"),
        }));

    assert!(!can_open);
    assert!(diagnostic.is_none());
}

#[test]
fn invalid_project_name_is_an_expected_disabled_open_state() {
    let (can_open, diagnostic) =
        project_probe_projection(Err(ProjectAuthorityError::ProjectName {
            source: ProjectNameError::Empty,
        }));

    assert!(!can_open);
    assert!(diagnostic.is_none());
}

#[test]
fn linked_path_probe_failure_remains_visible_at_the_ui_boundary() {
    let (can_open, diagnostic) = project_probe_projection(Err(ProjectAuthorityError::LinkedPath {
        path: PathBuf::from("linked-project"),
    }));

    assert!(!can_open);
    let diagnostic = diagnostic.expect("linked path must remain visible");
    assert!(diagnostic.contains("symbolic link or Windows reparse point"));
    assert_eq!(
        merge_probe_diagnostic("target is not empty".to_string(), Some(diagnostic)),
        "target is not empty; Existing project check failed: project path crosses a symbolic link or Windows reparse point: linked-project"
    );
}

#[test]
fn replacement_generation_rejects_a_late_completion_without_worker_timing_assumptions() {
    let mut state = WelcomeProjectProbeState::default();
    state.generation = 2;

    assert!(state
        .accept_completion(1, Ok(probe_result("stale-a")))
        .is_none());
    let (generation, result) = state
        .accept_completion(2, Ok(probe_result("current-b")))
        .expect("current generation should be accepted");
    assert_eq!(generation, 2);
    assert_eq!(result.unwrap().creation_validation, "current-b");
}

#[test]
fn first_draft_waits_for_the_probe_debounce_before_submission() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();

    state.request(draft("first"), now);

    assert!(!state
        .submit_due(
            &jobs,
            now + WELCOME_PROJECT_PROBE_DEBOUNCE - Duration::from_nanos(1),
        )
        .unwrap());
    assert!(state.active.is_none());
    assert!(state.pending.is_some());

    assert!(state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap());
    assert_eq!(state.active.as_ref().unwrap().generation, 1);
    state.clear();
}

#[test]
fn input_burst_submits_only_the_latest_draft_at_the_absolute_feedback_deadline() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();

    state.request(draft("first"), now);
    state.request(
        draft("latest"),
        now + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY - Duration::from_millis(1),
    );

    assert!(state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY)
        .unwrap());
    let active = state.active.as_ref().unwrap();
    assert_eq!(active.generation, 2);
    assert_eq!(active.draft.as_ref().unwrap().project_name, "latest");
    state.clear();
}

#[test]
fn repeated_same_draft_reuses_the_pending_generation() {
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();
    let draft = draft("same");

    state.request(draft.clone(), now);
    state.request(draft, now + Duration::from_millis(1));

    assert_eq!(state.generation, 1);
    assert_eq!(state.pending.as_ref().unwrap().generation, 1);
}

#[test]
fn repeated_same_draft_keeps_the_active_probe_ticket() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();
    let draft = draft("same-active");

    state.request(draft.clone(), now);
    state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap();
    let cancel = state.active.as_ref().unwrap().cancel.clone();

    state.request(
        draft,
        now + WELCOME_PROJECT_PROBE_DEBOUNCE + Duration::from_millis(1),
    );

    assert_eq!(state.generation, 1);
    assert!(!cancel.is_cancelled());
    assert!(state.active.is_some());
    state.clear();
}

#[test]
fn returning_to_the_active_draft_replaces_a_different_pending_draft() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();
    let active_draft = draft("active");

    state.request(active_draft.clone(), now);
    state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap();
    state.request(
        draft("replacement"),
        now + WELCOME_PROJECT_PROBE_DEBOUNCE + Duration::from_millis(1),
    );
    state.request(
        active_draft,
        now + WELCOME_PROJECT_PROBE_DEBOUNCE + Duration::from_millis(2),
    );

    assert_eq!(state.generation, 3);
    assert_eq!(state.pending.as_ref().unwrap().draft.project_name, "active");
    state.clear();
}

#[test]
fn due_replacement_merges_into_the_pending_probe_and_reuses_its_ticket() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Index, 1));
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let blocker = jobs
        .submit(
            EditorJobSpec::new("welcome-probe-blocker", JobCategory::Index),
            ProbeQueueGate {
                started: started_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let now = Instant::now();
    let mut state = WelcomeProjectProbeState::default();
    state.request(draft("first"), now);
    state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap();
    let first = state.active.as_ref().unwrap();
    let ticket_id = first.ticket.id();
    let stale_cancel = first.cancel.clone();

    state.request(
        draft("latest"),
        now + WELCOME_PROJECT_PROBE_DEBOUNCE + Duration::from_millis(1),
    );
    assert!(stale_cancel.is_cancelled());
    state
        .submit_due(
            &jobs,
            now + WELCOME_PROJECT_PROBE_DEBOUNCE * 2 + Duration::from_millis(1),
        )
        .unwrap();

    let active = state.active.as_ref().unwrap();
    assert_eq!(active.ticket.id(), ticket_id);
    assert_eq!(active.generation, 2);
    assert_eq!(active.draft.as_ref().unwrap().project_name, "latest");
    assert!(!active.cancel.is_cancelled());
    let snapshot = jobs.admission_snapshot();
    assert_eq!(snapshot.pending_entries(), 1);
    assert_eq!(snapshot.merged_submissions(), 1);
    assert!(snapshot.pending_estimated_bytes() >= "latest".len());

    state.clear();
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn clear_releases_the_pending_probe_reservation_before_key_rotation() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Index, 1));
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let blocker = jobs
        .submit(
            EditorJobSpec::new("welcome-clear-blocker", JobCategory::Index),
            ProbeQueueGate {
                started: started_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let now = Instant::now();
    let mut state = WelcomeProjectProbeState::default();
    state.request(draft("first"), now);
    state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap();
    let first_ticket = state.active.as_ref().unwrap().ticket.id();
    assert_eq!(jobs.admission_snapshot().pending_entries(), 1);

    state.clear();

    let after_clear = jobs.admission_snapshot();
    assert_eq!(after_clear.pending_entries(), 0);
    assert_eq!(after_clear.pending_estimated_bytes(), 0);
    assert_eq!(after_clear.cancelled_pending(), 1);

    state.request(draft("second"), now + Duration::from_millis(1));
    state
        .submit_due(
            &jobs,
            now + Duration::from_millis(1) + WELCOME_PROJECT_PROBE_DEBOUNCE,
        )
        .unwrap();
    assert_ne!(state.active.as_ref().unwrap().ticket.id(), first_ticket);
    assert_eq!(jobs.admission_snapshot().pending_entries(), 1);

    state.clear();
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn admission_backpressure_preserves_the_due_draft_for_retry() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            0,
            1,
            Duration::from_secs(1),
        )),
    );
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();
    state.request(draft("retry-me"), now);

    assert_eq!(
        state
            .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
            .unwrap_err(),
        JobSubmitError::AdmissionEntryLimitExceeded { limit: 0 }
    );
    let pending = state
        .pending
        .as_ref()
        .expect("backpressured draft must remain pending");
    assert_eq!(pending.generation, 1);
    assert_eq!(pending.draft.project_name, "retry-me");
    assert!(state.active.is_none());
}

#[test]
fn oldest_age_backpressure_retires_the_stale_ticket_and_preserves_latest_for_retry() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Index, 1));
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let blocker = jobs
        .submit(
            EditorJobSpec::new("welcome-age-blocker", JobCategory::Index),
            ProbeQueueGate {
                started: started_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let now = Instant::now();
    let mut state = WelcomeProjectProbeState::default();
    state.request(draft("stale"), now);
    state
        .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
        .unwrap();
    state.request(draft("latest"), now + Duration::from_millis(1));
    std::thread::sleep(WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY + Duration::from_millis(10));

    assert_eq!(
        state
            .submit_due(
                &jobs,
                now + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY + Duration::from_millis(20),
            )
            .unwrap_err(),
        JobSubmitError::OldestPendingAgeExceeded {
            max_age_ms: WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY.as_millis() as u64,
        }
    );
    assert!(state.active.is_none());
    assert_eq!(state.pending.as_ref().unwrap().draft.project_name, "latest");
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);

    assert!(state
        .submit_due(
            &jobs,
            now + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY + Duration::from_millis(21),
        )
        .unwrap());
    assert_eq!(
        state
            .active
            .as_ref()
            .unwrap()
            .draft
            .as_ref()
            .unwrap()
            .project_name,
        "latest"
    );
    assert_eq!(jobs.admission_snapshot().pending_entries(), 1);

    state.clear();
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn one_thousand_draft_burst_keeps_only_the_latest_probe_pending() {
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();

    for index in 0..1_000 {
        state.request(draft(&format!("draft-{index}")), now);
    }

    assert_eq!(state.generation, 1_000);
    assert!(state.active.is_none());
    assert_eq!(
        state.pending.as_ref().unwrap().draft.project_name,
        "draft-999"
    );
}

#[test]
fn one_million_draft_burst_keeps_only_the_latest_probe_pending() {
    let mut state = WelcomeProjectProbeState::default();
    let now = Instant::now();

    for index in 0..1_000_000 {
        state.request(draft(&format!("draft-{index}")), now);
    }

    assert_eq!(state.generation, 1_000_000);
    assert!(state.active.is_none());
    assert_eq!(
        state.pending.as_ref().unwrap().draft.project_name,
        "draft-999999"
    );
}

#[test]
fn cancellation_after_creation_validation_skips_the_authority_probe() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let probe_calls = Arc::new(AtomicUsize::new(0));
    state
        .submit_job(
            &jobs,
            CancellationBoundaryProbeJob {
                probe_calls: Arc::clone(&probe_calls),
            },
        )
        .unwrap();

    let (_generation, result) = poll_until_ready(&mut state);
    assert!(matches!(result, Err(JobError::Cancelled)));
    assert_eq!(probe_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn clear_is_observed_by_the_running_job_and_drops_the_active_probe() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (cancelled_sender, cancelled_receiver) = mpsc::sync_channel(1);
    state
        .submit_job(
            &jobs,
            CancellationProbeJob {
                started: started_sender,
                cancelled: cancelled_sender,
            },
        )
        .unwrap();
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("cancellation probe should start");
    let cancel = state.active.as_ref().unwrap().cancel.clone();

    state.clear();

    cancelled_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("running job should observe cancellation");
    assert!(cancel.is_cancelled());
    assert!(state.active.is_none());
    assert!(state.poll().is_none());
}

#[test]
fn job_and_submit_failures_remain_typed_at_the_probe_boundary() {
    let jobs = test_job_system();
    let mut state = WelcomeProjectProbeState::default();
    state
        .submit_job(&jobs, ControlledProbeJob::failed())
        .unwrap();
    let (_generation, result) = poll_until_ready(&mut state);
    let error = match result {
        Ok(_) => panic!("job failure should remain visible"),
        Err(error) => error,
    };
    assert!(error.downcast_ref::<io::Error>().is_some());

    let stopped_jobs = test_job_system();
    stopped_jobs.shutdown(Instant::now());
    let submit_error = state
        .submit_job(&stopped_jobs, ControlledProbeJob::immediate("rejected"))
        .unwrap_err();
    assert_eq!(submit_error, JobSubmitError::ShuttingDown);
    assert!(state.active.is_none());
}

struct ControlledProbeJob {
    validation: &'static str,
    fail: bool,
}

impl ControlledProbeJob {
    fn immediate(validation: &'static str) -> Self {
        Self {
            validation,
            fail: false,
        }
    }

    fn failed() -> Self {
        Self {
            validation: "failed",
            fail: true,
        }
    }
}

impl EditorJob for ControlledProbeJob {
    type Output = WelcomeProjectProbeResult;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        if self.fail {
            return Err(JobError::failed(io::Error::other("planned probe failure")));
        }
        Ok(probe_result(self.validation))
    }
}

struct CancellationProbeJob {
    started: mpsc::SyncSender<()>,
    cancelled: mpsc::SyncSender<()>,
}

struct CancellationBoundaryProbeJob {
    probe_calls: Arc<AtomicUsize>,
}

struct ProbeQueueGate {
    started: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

impl EditorJob for ProbeQueueGate {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv().map_err(JobError::failed)
    }
}

impl EditorJob for CancellationBoundaryProbeJob {
    type Output = WelcomeProjectProbeResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        WelcomeProjectProbeJob {
            draft: draft("cancel-between-phases"),
        }
        .run_with(
            &context,
            |_| {
                context.cancellation_token().cancel();
                String::new()
            },
            |_| {
                self.probe_calls.fetch_add(1, Ordering::Relaxed);
                unreachable!("cancelled probes must not reach ProjectAuthority")
            },
        )
    }
}

impl EditorJob for CancellationProbeJob {
    type Output = WelcomeProjectProbeResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        while !context.is_cancelled() {
            std::thread::yield_now();
        }
        self.cancelled.send(()).unwrap();
        context.check_cancelled()?;
        unreachable!("cancelled context must return before producing a result")
    }
}

fn probe_result(validation: &'static str) -> WelcomeProjectProbeResult {
    WelcomeProjectProbeResult {
        creation_validation: validation.to_string(),
        open_probe: Err(ProjectAuthorityError::ProjectMissing {
            path: PathBuf::from(validation),
        }),
    }
}

fn draft(project_name: &str) -> crate::core::project::NewProjectDraft {
    crate::core::project::NewProjectDraft {
        project_name: project_name.to_string(),
        location: "projects".to_string(),
        ..Default::default()
    }
}

fn poll_until_ready(
    state: &mut WelcomeProjectProbeState,
) -> (u64, Result<WelcomeProjectProbeResult, JobError>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = state.poll() {
            return result;
        }
        assert!(Instant::now() < deadline, "probe result timed out");
        std::thread::yield_now();
    }
}
