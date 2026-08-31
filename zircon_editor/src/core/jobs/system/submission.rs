use std::sync::mpsc;
use std::time::Instant;

use super::pending::PendingJob;
use super::pending_task::{LatestPendingTask, PendingTask};
use super::{
    EditorJobAdmissionWindow, EditorJobBatchAdmissionReservation, EditorJobSystem,
    ProgressObserverEvent,
};
use crate::core::jobs::{
    EditorJob, EditorJobAdmission, EditorJobAdmissionRequest, EditorJobSpec, JobContext, JobError,
    JobEventKind, JobSubmitError, JobTicket,
};

impl EditorJobSystem {
    pub fn submit<J>(
        &self,
        spec: EditorJobSpec,
        job: J,
    ) -> Result<JobTicket<J::Output>, JobSubmitError>
    where
        J: EditorJob,
    {
        if spec.admission_key.is_some() {
            return Err(JobSubmitError::KeyedAdmissionRequiresOutcome);
        }
        match self.submit_admitted(spec, job)? {
            EditorJobAdmission::Accepted(ticket) => Ok(ticket),
            EditorJobAdmission::Merged { .. } => {
                unreachable!("unkeyed editor jobs cannot merge into a pending reservation")
            }
        }
    }

    pub fn submit_admitted<J>(
        &self,
        spec: EditorJobSpec,
        job: J,
    ) -> Result<EditorJobAdmission<J::Output>, JobSubmitError>
    where
        J: EditorJob,
    {
        if spec.label.trim().is_empty() {
            return Err(JobSubmitError::EmptyLabel);
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancel_sender = sender.clone();
        let mut task: Option<Box<dyn PendingTask>> =
            Some(Box::new(LatestPendingTask::new(job, sender)));
        let cancel_task = Box::new(move |context: JobContext| {
            context.emit(JobEventKind::Cancelled);
            let _ = cancel_sender.send(Err(JobError::Cancelled));
        });

        let id = {
            let mut state = self.inner.lock_state();
            state.ensure_accepting_submissions()?;
            let admitted_at = Instant::now();
            if let Some(existing_job) = state.pending_admission_id(&spec) {
                let existing_job = state.merge_pending_admission(
                    existing_job,
                    &spec,
                    task.take().expect("pending task exists before keyed merge"),
                    &self.inner.limits,
                    admitted_at,
                )?;
                self.inner.progress.register(existing_job, &spec);
                return Ok(EditorJobAdmission::Merged { existing_job });
            }
            for dependency in &spec.after {
                state.validate_dependency(*dependency)?;
            }
            state.ensure_pending_admissible(&spec, &self.inner.limits, admitted_at)?;
            let id = state.allocate_id();
            state.register(id);
            self.inner.progress.register(id, &spec);
            state.enqueue_pending(PendingJob::new(
                id,
                spec,
                task.take().expect("pending task exists before admission"),
                cancel_task,
                admitted_at,
            ));
            self.inner
                .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(id));
            id
        };
        self.inner.deliver_progress_observer_events();
        self.inner.promote();
        Ok(EditorJobAdmission::Accepted(JobTicket::new(id, receiver)))
    }

    /// Admits an all-or-nothing group of unkeyed jobs through the one queue.
    ///
    /// This is for a single logical operation whose callers must not observe a
    /// partial set of queued work after admission backpressure. Individual
    /// keyed requests use [`Self::submit_admitted`] so their merge outcome
    /// remains explicit.
    pub fn submit_batch<J>(
        &self,
        requests: Vec<(EditorJobSpec, J)>,
    ) -> Result<Vec<JobTicket<J::Output>>, JobSubmitError>
    where
        J: EditorJob,
    {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        for (spec, _) in &requests {
            if spec.label.trim().is_empty() {
                return Err(JobSubmitError::EmptyLabel);
            }
            if spec.admission_key.is_some() {
                return Err(JobSubmitError::KeyedAdmissionRequiresOutcome);
            }
        }

        let mut submissions = requests
            .into_iter()
            .map(|(spec, job)| {
                let (sender, receiver) = mpsc::sync_channel(1);
                let cancel_sender = sender.clone();
                let task: Box<dyn PendingTask> = Box::new(LatestPendingTask::new(job, sender));
                let cancel_task: Box<dyn FnOnce(JobContext) + Send + 'static> =
                    Box::new(move |context: JobContext| {
                        context.emit(JobEventKind::Cancelled);
                        let _ = cancel_sender.send(Err(JobError::Cancelled));
                    });
                (spec, task, cancel_task, receiver)
            })
            .collect::<Vec<_>>();

        let tickets = {
            let mut state = self.inner.lock_state();
            state.ensure_accepting_submissions()?;
            let admitted_at = Instant::now();
            let specs = submissions
                .iter()
                .map(|(spec, _, _, _)| spec)
                .collect::<Vec<_>>();
            state.ensure_batch_pending_admissible(&specs, &self.inner.limits, admitted_at)?;
            for spec in &specs {
                for dependency in &spec.after {
                    state.validate_dependency(*dependency)?;
                }
            }

            let mut tickets = Vec::with_capacity(submissions.len());
            for (spec, task, cancel_task, receiver) in submissions.drain(..) {
                let id = state.allocate_id();
                state.register(id);
                self.inner.progress.register(id, &spec);
                state.enqueue_pending(PendingJob::new(id, spec, task, cancel_task, admitted_at));
                self.inner
                    .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(id));
                tickets.push(JobTicket::new(id, receiver));
            }
            tickets
        };
        self.inner.deliver_progress_observer_events();
        self.inner.promote();
        Ok(tickets)
    }

    /// Atomically claims pending capacity before materializing worker resources.
    ///
    /// The returned reservation is intentionally non-executable: callers can
    /// resolve resource mutexes and construct job payloads after this succeeds,
    /// then either commit matching jobs or drop it to roll capacity back.
    pub fn reserve_batch_admission(
        &self,
        requests: Vec<EditorJobAdmissionRequest>,
    ) -> Result<EditorJobBatchAdmissionReservation, JobSubmitError> {
        let reservation_id = self.inner.lock_state().reserve_batch_admission(
            requests,
            &self.inner.limits,
            Instant::now(),
        )?;
        Ok(EditorJobBatchAdmissionReservation::new(
            self.clone(),
            reservation_id,
        ))
    }

    /// Checks the shared entry, estimated-byte, and oldest-age budgets without
    /// allocating a job id, result channel, or pending task.
    pub fn pending_admission_window(&self) -> Result<EditorJobAdmissionWindow, JobSubmitError> {
        self.inner
            .lock_state()
            .pending_admission_window(&self.inner.limits, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::time::Duration;

    use crate::core::jobs::{
        test_job_system_with_limits, CancellationToken, EditorJob, EditorJobAdmission,
        EditorJobAdmissionKey, EditorJobAdmissionLimits, EditorJobLimits, EditorJobSpec,
        JobCategory, JobContext, JobError,
    };

    struct GateJob {
        started: Sender<()>,
        release: Receiver<()>,
    }

    impl EditorJob for GateJob {
        type Output = ();

        fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
            let _ = self.started.send(());
            self.release.recv().map_err(JobError::failed)
        }
    }

    struct ValueJob(u32);

    impl EditorJob for ValueJob {
        type Output = u32;

        fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
            Ok(self.0)
        }
    }

    #[test]
    fn keyed_pending_merge_refreshes_progress_cancellation_authority() {
        let jobs = test_job_system_with_limits(
            EditorJobLimits::default()
                .with_limit(JobCategory::Export, 1)
                .with_admission_limits(EditorJobAdmissionLimits::new(
                    4,
                    32,
                    Duration::from_secs(60),
                )),
        );
        let (blocker_started, blocker_started_receiver) = mpsc::channel();
        let (release_blocker, release_blocker_receiver) = mpsc::channel();
        let blocker = jobs
            .submit(
                EditorJobSpec::new("merge-cancel-blocker", JobCategory::Export),
                GateJob {
                    started: blocker_started,
                    release: release_blocker_receiver,
                },
            )
            .unwrap();
        blocker_started_receiver.recv().unwrap();

        let key = EditorJobAdmissionKey::new("merge-cancel-authority").unwrap();
        let stale_cancel = CancellationToken::default();
        let accepted = jobs
            .submit_admitted(
                EditorJobSpec::new("merge-cancel-first", JobCategory::Export)
                    .with_estimated_bytes(8)
                    .with_cancel(stale_cancel.clone())
                    .with_admission_key(key.clone()),
                ValueJob(1),
            )
            .unwrap();
        let accepted = match accepted {
            EditorJobAdmission::Accepted(ticket) => ticket,
            EditorJobAdmission::Merged { .. } => panic!("first keyed request must reserve a job"),
        };

        let current_cancel = CancellationToken::default();
        let merged = jobs
            .submit_admitted(
                EditorJobSpec::new("merge-cancel-latest", JobCategory::Export)
                    .with_estimated_bytes(8)
                    .with_cancel(current_cancel.clone())
                    .with_admission_key(key),
                ValueJob(2),
            )
            .unwrap();
        assert!(matches!(
            merged,
            EditorJobAdmission::Merged { existing_job } if existing_job == accepted.id()
        ));

        assert!(jobs.inner.progress.request_cancel(accepted.id()));
        assert!(current_cancel.is_cancelled());
        assert!(!stale_cancel.is_cancelled());

        assert!(jobs.cancel(accepted.id()));
        release_blocker.send(()).unwrap();
        assert_eq!(blocker.wait(), Ok(()));
        assert_eq!(accepted.wait(), Err(JobError::Cancelled));
    }
}
