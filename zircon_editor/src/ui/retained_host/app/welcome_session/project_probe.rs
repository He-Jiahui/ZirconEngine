use super::super::*;
use crate::core::jobs::{
    CancellationToken, EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority,
    JobTicket,
};
use crate::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectAuthorityError, ProjectProbe,
};
use std::time::{Duration, Instant};

// Keep validation responsive while ensuring an uninterrupted input burst cannot defer feedback.
const WELCOME_PROJECT_PROBE_DEBOUNCE: Duration = Duration::from_millis(50);
const WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY: Duration = Duration::from_millis(250);

pub(in crate::ui::retained_host::app) struct WelcomeProjectProbeState {
    generation: u64,
    pending: Option<PendingWelcomeProjectProbe>,
    active: Option<ActiveWelcomeProjectProbe>,
}

struct PendingWelcomeProjectProbe {
    generation: u64,
    draft: NewProjectDraft,
    first_requested_at: Instant,
    last_requested_at: Instant,
}

struct ActiveWelcomeProjectProbe {
    generation: u64,
    draft: Option<NewProjectDraft>,
    cancel: CancellationToken,
    ticket: JobTicket<WelcomeProjectProbeResult>,
}

struct WelcomeProjectProbeJob {
    draft: NewProjectDraft,
}

struct WelcomeProjectProbeResult {
    creation_validation: String,
    open_probe: Result<ProjectProbe, ProjectAuthorityError>,
}

impl Default for WelcomeProjectProbeState {
    fn default() -> Self {
        Self {
            generation: 0,
            pending: None,
            active: None,
        }
    }
}

impl WelcomeProjectProbeState {
    fn request(&mut self, draft: NewProjectDraft, now: Instant) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.draft == draft)
            || self
                .active
                .as_ref()
                .is_some_and(|active| active.draft.as_ref() == Some(&draft))
        {
            return;
        }

        self.clear_active();
        self.generation = self.generation.wrapping_add(1);
        let first_requested_at = self
            .pending
            .as_ref()
            .map_or(now, |pending| pending.first_requested_at);
        self.pending = Some(PendingWelcomeProjectProbe {
            generation: self.generation,
            draft,
            first_requested_at,
            last_requested_at: now,
        });
    }

    fn submit_due(
        &mut self,
        jobs: &crate::core::jobs::EditorJobSystem,
        now: Instant,
    ) -> Result<bool, crate::core::jobs::JobSubmitError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(false);
        };
        let debounce_due = pending.last_requested_at + WELCOME_PROJECT_PROBE_DEBOUNCE;
        let maximum_due = pending.first_requested_at + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY;
        if now < debounce_due && now < maximum_due {
            return Ok(false);
        }

        let pending = self
            .pending
            .take()
            .expect("pending probe exists after due check");
        let draft = pending.draft;
        self.submit_generation(
            jobs,
            pending.generation,
            Some(draft.clone()),
            WelcomeProjectProbeJob { draft },
        )?;
        Ok(true)
    }

    fn submit_job<J>(
        &mut self,
        jobs: &crate::core::jobs::EditorJobSystem,
        job: J,
    ) -> Result<(), crate::core::jobs::JobSubmitError>
    where
        J: EditorJob<Output = WelcomeProjectProbeResult>,
    {
        self.clear();
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.submit_generation(jobs, generation, None, job)
    }

    fn submit_generation<J>(
        &mut self,
        jobs: &crate::core::jobs::EditorJobSystem,
        generation: u64,
        draft: Option<NewProjectDraft>,
        job: J,
    ) -> Result<(), crate::core::jobs::JobSubmitError>
    where
        J: EditorJob<Output = WelcomeProjectProbeResult>,
    {
        let cancel = CancellationToken::default();
        let ticket = jobs.submit(
            EditorJobSpec::new("Probe welcome project", JobCategory::Index)
                .with_priority(JobPriority::Background)
                .with_cancel(cancel.clone()),
            job,
        )?;
        self.active = Some(ActiveWelcomeProjectProbe {
            generation,
            draft,
            cancel,
            ticket,
        });
        Ok(())
    }

    fn poll(&mut self) -> Option<(u64, Result<WelcomeProjectProbeResult, JobError>)> {
        let result = self.active.as_ref()?.ticket.try_take()?;
        let active = self
            .active
            .take()
            .expect("active probe exists while polling");
        self.accept_completion(active.generation, result)
    }

    fn accept_completion(
        &self,
        generation: u64,
        result: Result<WelcomeProjectProbeResult, JobError>,
    ) -> Option<(u64, Result<WelcomeProjectProbeResult, JobError>)> {
        (generation == self.generation).then_some((generation, result))
    }

    fn clear(&mut self) {
        self.pending = None;
        self.clear_active();
    }

    fn clear_active(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancel.cancel();
        }
    }
}

impl Drop for WelcomeProjectProbeState {
    fn drop(&mut self) {
        self.clear();
    }
}

impl EditorJob for WelcomeProjectProbeJob {
    type Output = WelcomeProjectProbeResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        self.run_with(
            &context,
            |draft| {
                draft
                    .validate_for_creation()
                    .map(|_| String::new())
                    .unwrap_or_else(|error| error.to_string())
            },
            |draft| ProjectAuthority::default().probe_draft(draft),
        )
    }
}

impl WelcomeProjectProbeJob {
    fn run_with<V, P>(
        &self,
        context: &JobContext,
        validate_creation: V,
        probe_existing: P,
    ) -> Result<WelcomeProjectProbeResult, JobError>
    where
        V: FnOnce(&NewProjectDraft) -> String,
        P: FnOnce(&NewProjectDraft) -> Result<ProjectProbe, ProjectAuthorityError>,
    {
        context.check_cancelled()?;
        let creation_validation = validate_creation(&self.draft);
        context.check_cancelled()?;
        let open_probe = probe_existing(&self.draft);
        context.check_cancelled()?;
        Ok(WelcomeProjectProbeResult {
            creation_validation,
            open_probe,
        })
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn schedule_welcome_project_probe(&mut self) {
        self.startup_session.creation_validation = "Checking project location…".to_string();
        self.startup_session.can_open_existing = false;
        self.welcome_project_probe
            .request(self.startup_session.draft.clone(), Instant::now());
    }

    pub(super) fn clear_welcome_project_probe(&mut self) {
        self.welcome_project_probe.clear();
    }

    pub(in crate::ui::retained_host::app) fn poll_welcome_project_probe(&mut self) {
        if let Err(error) = self
            .welcome_project_probe
            .submit_due(self.editor_manager.context().jobs(), Instant::now())
        {
            self.startup_session.creation_validation =
                format!("Project validation is unavailable: {error}");
            self.startup_session.can_open_existing = false;
            self.refresh_welcome_snapshot();
            return;
        }
        let Some((_generation, result)) = self.welcome_project_probe.poll() else {
            return;
        };
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                self.startup_session.creation_validation =
                    format!("Project validation job failed: {error}");
                self.startup_session.can_open_existing = false;
                self.refresh_welcome_snapshot();
                return;
            }
        };
        let (can_open_existing, open_diagnostic) = project_probe_projection(result.open_probe);
        self.startup_session.creation_validation =
            merge_probe_diagnostic(result.creation_validation, open_diagnostic);
        self.startup_session.can_open_existing = can_open_existing;
        self.refresh_welcome_snapshot();
    }
}

fn project_probe_projection(
    result: Result<ProjectProbe, ProjectAuthorityError>,
) -> (bool, Option<String>) {
    match result {
        Ok(_) => (true, None),
        Err(
            ProjectAuthorityError::ProjectName { .. }
            | ProjectAuthorityError::EmptyProjectLocation
            | ProjectAuthorityError::ProjectMissing { .. }
            | ProjectAuthorityError::ManifestMissing { .. },
        ) => (false, None),
        Err(error) => (
            false,
            Some(format!("Existing project check failed: {error}")),
        ),
    }
}

fn merge_probe_diagnostic(creation_validation: String, open_diagnostic: Option<String>) -> String {
    let Some(open_diagnostic) = open_diagnostic else {
        return creation_validation;
    };
    if creation_validation.is_empty() {
        open_diagnostic
    } else {
        format!("{creation_validation}; {open_diagnostic}")
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    };
    use std::time::{Duration, Instant};

    use super::{
        WELCOME_PROJECT_PROBE_DEBOUNCE, WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY,
        WelcomeProjectProbeJob, WelcomeProjectProbeResult, WelcomeProjectProbeState,
        merge_probe_diagnostic, project_probe_projection,
    };
    use crate::core::jobs::{EditorJob, JobContext, JobError, JobSubmitError, test_job_system};
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
        let (can_open, diagnostic) =
            project_probe_projection(Err(ProjectAuthorityError::LinkedPath {
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

        assert!(
            state
                .accept_completion(1, Ok(probe_result("stale-a")))
                .is_none()
        );
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

        assert!(
            !state
                .submit_due(
                    &jobs,
                    now + WELCOME_PROJECT_PROBE_DEBOUNCE - Duration::from_nanos(1),
                )
                .unwrap()
        );
        assert!(state.active.is_none());
        assert!(state.pending.is_some());

        assert!(
            state
                .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_DEBOUNCE)
                .unwrap()
        );
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

        assert!(
            state
                .submit_due(&jobs, now + WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY)
                .unwrap()
        );
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
}
