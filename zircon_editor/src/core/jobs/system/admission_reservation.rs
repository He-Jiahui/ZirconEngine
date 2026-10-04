use std::sync::mpsc;

use super::pending::PendingJob;
use super::pending_task::{LatestPendingTask, PendingTask};
use super::{EditorJobSystem, ProgressObserverEvent};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobContext, JobError, JobEventKind, JobSubmitError, JobTicket,
};

/// Owns a successful pending-admission claim until its jobs are materialized.
///
/// Dropping an uncommitted reservation atomically releases its queue entries
/// and estimated bytes. The reservation does not create jobs, result channels,
/// or scheduler work by itself.
pub struct EditorJobBatchAdmissionReservation {
    jobs: EditorJobSystem,
    reservation_id: Option<u64>,
}

impl EditorJobBatchAdmissionReservation {
    pub(super) fn new(jobs: EditorJobSystem, reservation_id: u64) -> Self {
        Self {
            jobs,
            reservation_id: Some(reservation_id),
        }
    }

    /// Materializes and queues the exact number of jobs covered by this claim.
    pub fn commit<J>(
        mut self,
        requests: Vec<(EditorJobSpec, J)>,
    ) -> Result<Vec<JobTicket<J::Output>>, JobSubmitError>
    where
        J: EditorJob,
    {
        if requests.is_empty() {
            return Err(JobSubmitError::AdmissionReservationMismatch);
        }
        for (spec, _) in &requests {
            if spec.label.trim().is_empty() || spec.admission_key.is_some() {
                return Err(JobSubmitError::AdmissionReservationMismatch);
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
        let reservation_id = self
            .reservation_id
            .expect("batch admission reservations commit at most once");
        let specs = submissions
            .iter()
            .map(|(spec, _, _, _)| spec)
            .collect::<Vec<_>>();
        let tickets = {
            let mut state = self.jobs.inner.lock_state();
            let reserved = state.commit_batch_admission_reservation(reservation_id, &specs)?;
            drop(specs);
            let mut tickets = Vec::with_capacity(submissions.len());
            for ((id, admitted_at), (spec, task, cancel_task, receiver)) in
                reserved.into_iter().zip(submissions.drain(..))
            {
                state.register(id);
                self.jobs.inner.progress.register(id, &spec);
                state.enqueue_pending(PendingJob::new(id, spec, task, cancel_task, admitted_at));
                self.jobs
                    .inner
                    .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(id));
                tickets.push(JobTicket::new(id, receiver));
            }
            tickets
        };
        self.reservation_id = None;
        self.jobs.inner.deliver_progress_observer_events();
        self.jobs.inner.promote();
        Ok(tickets)
    }
}

impl Drop for EditorJobBatchAdmissionReservation {
    fn drop(&mut self) {
        let Some(reservation_id) = self.reservation_id.take() else {
            return;
        };
        self.jobs
            .inner
            .lock_state()
            .release_batch_admission_reservation(reservation_id);
    }
}

#[cfg(test)]
#[path = "tests/admission_reservation.rs"]
mod tests;
