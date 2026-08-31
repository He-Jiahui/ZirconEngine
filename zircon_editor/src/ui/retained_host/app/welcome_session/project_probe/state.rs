use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::jobs::{
    CancellationToken, EditorJob, EditorJobAdmission, EditorJobAdmissionKey, EditorJobSpec,
    EditorJobSystem, JobCategory, JobError, JobPriority, JobTicket, MutexGroup,
};
use crate::core::project::NewProjectDraft;

use super::job::{WelcomeProjectProbeJob, WelcomeProjectProbeResult};

pub(super) const WELCOME_PROJECT_PROBE_DEBOUNCE: Duration = Duration::from_millis(50);
pub(super) const WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY: Duration = Duration::from_millis(250);
static NEXT_WELCOME_PROJECT_PROBE_OWNER: AtomicU64 = AtomicU64::new(1);

pub(in crate::ui::retained_host::app) struct WelcomeProjectProbeState {
    pub(super) generation: u64,
    pub(super) pending: Option<PendingWelcomeProjectProbe>,
    pub(super) active: Option<ActiveWelcomeProjectProbe>,
    admission_key: EditorJobAdmissionKey,
    mutex_group: MutexGroup,
}

pub(super) struct PendingWelcomeProjectProbe {
    pub(super) generation: u64,
    pub(super) draft: NewProjectDraft,
    first_requested_at: Instant,
    last_requested_at: Instant,
}

pub(super) struct ActiveWelcomeProjectProbe {
    pub(super) generation: u64,
    pub(super) draft: Option<NewProjectDraft>,
    pub(super) cancel: CancellationToken,
    jobs: EditorJobSystem,
    pub(super) ticket: JobTicket<WelcomeProjectProbeResult>,
}

impl Default for WelcomeProjectProbeState {
    fn default() -> Self {
        let owner = next_welcome_project_probe_owner();
        Self {
            generation: 0,
            pending: None,
            active: None,
            admission_key: welcome_project_probe_admission_key(owner),
            mutex_group: welcome_project_probe_mutex_group(owner),
        }
    }
}

impl WelcomeProjectProbeState {
    pub(super) fn request(&mut self, draft: NewProjectDraft, now: Instant) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.draft == draft)
            || (self.pending.is_none()
                && self
                    .active
                    .as_ref()
                    .is_some_and(|active| active.draft.as_ref() == Some(&draft)))
        {
            return;
        }

        if let Some(active) = self.active.as_ref() {
            active.cancel.cancel();
        }
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

    pub(super) fn submit_due(
        &mut self,
        jobs: &EditorJobSystem,
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

        let (generation, draft) = {
            let pending = self
                .pending
                .as_ref()
                .expect("pending probe exists after due check");
            (pending.generation, pending.draft.clone())
        };
        if let Err(error) = self.submit_generation(
            jobs,
            generation,
            Some(draft.clone()),
            WelcomeProjectProbeJob { draft },
        ) {
            if matches!(
                error,
                crate::core::jobs::JobSubmitError::OldestPendingAgeExceeded { .. }
            ) {
                self.clear_active();
                self.rotate_admission_key();
            }
            return Err(error);
        }
        self.pending = None;
        Ok(true)
    }

    pub(super) fn submit_job<J>(
        &mut self,
        jobs: &EditorJobSystem,
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
        jobs: &EditorJobSystem,
        generation: u64,
        draft: Option<NewProjectDraft>,
        job: J,
    ) -> Result<(), crate::core::jobs::JobSubmitError>
    where
        J: EditorJob<Output = WelcomeProjectProbeResult>,
    {
        let cancel = CancellationToken::default();
        let estimated_pending_bytes = draft.as_ref().map_or(1, welcome_project_probe_bytes);
        let admission = jobs.submit_admitted(
            EditorJobSpec::new("Probe welcome project", JobCategory::Index)
                .with_priority(JobPriority::Background)
                .with_mutex_group(self.mutex_group.clone())
                .with_cancel(cancel.clone())
                .with_estimated_bytes(estimated_pending_bytes)
                .with_admission_key(self.admission_key.clone())
                .with_max_pending_age(WELCOME_PROJECT_PROBE_MAX_FEEDBACK_DELAY),
            job,
        )?;
        match admission {
            EditorJobAdmission::Accepted(ticket) => {
                self.active = Some(ActiveWelcomeProjectProbe {
                    generation,
                    draft,
                    cancel,
                    jobs: jobs.clone(),
                    ticket,
                });
            }
            EditorJobAdmission::Merged { existing_job } => {
                let Some(active) = self
                    .active
                    .as_mut()
                    .filter(|active| active.ticket.id() == existing_job)
                else {
                    jobs.cancel(existing_job);
                    return Err(crate::core::jobs::JobSubmitError::AdmissionKeyConflict {
                        existing_job,
                    });
                };
                active.generation = generation;
                active.draft = draft;
                active.cancel = cancel;
            }
        }
        Ok(())
    }

    pub(super) fn poll(&mut self) -> Option<(u64, Result<WelcomeProjectProbeResult, JobError>)> {
        let result = self.active.as_ref()?.ticket.try_take()?;
        let active = self
            .active
            .take()
            .expect("active probe exists while polling");
        self.accept_completion(active.generation, result)
    }

    pub(super) fn accept_completion(
        &self,
        generation: u64,
        result: Result<WelcomeProjectProbeResult, JobError>,
    ) -> Option<(u64, Result<WelcomeProjectProbeResult, JobError>)> {
        (generation == self.generation).then_some((generation, result))
    }

    pub(super) fn clear(&mut self) {
        self.pending = None;
        self.clear_active();
        self.rotate_admission_key();
    }

    fn clear_active(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancel.cancel();
            active.jobs.cancel(active.ticket.id());
        }
    }

    fn rotate_admission_key(&mut self) {
        self.admission_key =
            welcome_project_probe_admission_key(next_welcome_project_probe_owner());
    }
}

impl Drop for WelcomeProjectProbeState {
    fn drop(&mut self) {
        self.pending = None;
        self.clear_active();
    }
}

fn next_welcome_project_probe_owner() -> u64 {
    NEXT_WELCOME_PROJECT_PROBE_OWNER.fetch_add(1, Ordering::Relaxed)
}

fn welcome_project_probe_admission_key(owner: u64) -> EditorJobAdmissionKey {
    EditorJobAdmissionKey::new(format!("welcome-project-probe:{owner}"))
        .expect("built-in welcome project admission key must remain valid")
}

fn welcome_project_probe_mutex_group(owner: u64) -> MutexGroup {
    MutexGroup::parse(format!("welcome_project_probe_{owner}"))
        .expect("built-in welcome project mutex group must remain valid")
}

fn welcome_project_probe_bytes(draft: &NewProjectDraft) -> usize {
    size_of::<WelcomeProjectProbeJob>()
        .saturating_add(draft.project_name.capacity())
        .saturating_add(draft.location.capacity())
}
