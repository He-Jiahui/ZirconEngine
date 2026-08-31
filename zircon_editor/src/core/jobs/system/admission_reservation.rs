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
mod tests {
    use std::hint::black_box;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::time::Instant;

    use crate::core::jobs::{
        test_job_system_with_limits, EditorJob, EditorJobAdmissionLimits,
        EditorJobAdmissionRequest, EditorJobLimits, EditorJobSpec, JobCategory, JobContext,
        JobError, JobSubmitError,
    };

    struct ValueJob(u32);

    impl EditorJob for ValueJob {
        type Output = u32;

        fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
            Ok(self.0 + 1)
        }
    }

    #[test]
    fn rejected_reservation_does_not_consume_a_job_id() {
        let jobs =
            test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
                EditorJobAdmissionLimits::new(1, 8, Duration::from_secs(60)),
            ));

        assert!(matches!(
            jobs.reserve_batch_admission(vec![
                EditorJobAdmissionRequest::new(JobCategory::InteractiveSave, 8),
                EditorJobAdmissionRequest::new(JobCategory::InteractiveSave, 8),
            ]),
            Err(JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 })
        ));

        let ticket = jobs
            .submit(
                EditorJobSpec::new("first-accepted-job", JobCategory::InteractiveSave),
                ValueJob(1),
            )
            .unwrap();
        assert_eq!(ticket.id().value(), 1);
        assert_eq!(ticket.wait(), Ok(2));
    }

    #[test]
    fn rejected_commit_releases_its_pending_admission_capacity() {
        let jobs =
            test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
                EditorJobAdmissionLimits::new(1, 8, Duration::from_secs(60)),
            ));
        let reservation = jobs
            .reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
                JobCategory::InteractiveSave,
                8,
            )])
            .unwrap();

        assert!(matches!(
            reservation.commit(vec![(
                EditorJobSpec::new("mismatched-reservation", JobCategory::InteractiveSave)
                    .with_estimated_bytes(16),
                ValueJob(1),
            )]),
            Err(JobSubmitError::AdmissionReservationMismatch)
        ));

        let ticket = jobs
            .submit(
                EditorJobSpec::new("capacity-released", JobCategory::InteractiveSave)
                    .with_estimated_bytes(8),
                ValueJob(1),
            )
            .unwrap();
        assert_eq!(ticket.wait(), Ok(2));
    }

    #[test]
    fn optimization_batch_fo_editor401_collects_batch_specs_before_locking_state() {
        let source = include_str!("admission_reservation.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("admission reservation production source");
        let specs = production
            .find("let specs = submissions")
            .expect("batch specs must be collected once");
        let lock = production
            .find("let mut state = self.jobs.inner.lock_state()")
            .expect("batch commit must lock scheduler state");

        assert!(specs < lock);
        assert!(production.contains("drop(specs);"));
        assert_eq!(production.matches("collect::<Vec<_>>()").count(), 2);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fo_editor401_precomputed_batch_specs_lock_hold_benchmark() {
        const SAMPLE_PAIRS: usize = 17;
        const SPEC_COUNT: usize = 128;
        const COLLECTIONS_PER_SAMPLE: usize = 4_096;

        let specs = (0..SPEC_COUNT)
            .map(|index| {
                EditorJobSpec::new(
                    format!("optimization-batch-fo-job-{index:03}"),
                    JobCategory::InteractiveSave,
                )
            })
            .collect::<Vec<_>>();
        let state_lock = Mutex::new(());

        for _ in 0..4 {
            black_box(measure_legacy_lock_hold(
                &state_lock,
                &specs,
                COLLECTIONS_PER_SAMPLE,
            ));
            black_box(measure_optimized_lock_hold(
                &state_lock,
                &specs,
                COLLECTIONS_PER_SAMPLE,
            ));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_lock_hold(
                    &state_lock,
                    &specs,
                    COLLECTIONS_PER_SAMPLE,
                ));
                optimized_samples.push(measure_optimized_lock_hold(
                    &state_lock,
                    &specs,
                    COLLECTIONS_PER_SAMPLE,
                ));
            } else {
                optimized_samples.push(measure_optimized_lock_hold(
                    &state_lock,
                    &specs,
                    COLLECTIONS_PER_SAMPLE,
                ));
                legacy_samples.push(measure_legacy_lock_hold(
                    &state_lock,
                    &specs,
                    COLLECTIONS_PER_SAMPLE,
                ));
            }
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR401_PRECOMPUTED_BATCH_SPECS_LOCK_HOLD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} spec_count={SPEC_COUNT} collections_per_sample={COLLECTIONS_PER_SAMPLE} legacy_allocations_under_lock_per_sample={COLLECTIONS_PER_SAMPLE} optimized_allocations_under_lock_per_sample=0 deallocations_under_lock_per_sample={COLLECTIONS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
            "precomputed batch specs must reduce isolated lock-hold P95 by at least 25%"
        );
    }

    fn measure_legacy_lock_hold(
        state_lock: &Mutex<()>,
        specs: &[EditorJobSpec],
        collections: usize,
    ) -> u128 {
        let mut held_ns = 0_u128;
        let mut checksum = 0_usize;
        for _ in 0..collections {
            let guard = state_lock.lock().unwrap();
            let started = Instant::now();
            let collected = black_box(specs).iter().collect::<Vec<_>>();
            checksum = checksum.wrapping_add(collected.len());
            black_box(&collected);
            drop(collected);
            held_ns = held_ns.saturating_add(started.elapsed().as_nanos());
            drop(guard);
        }
        black_box(checksum);
        held_ns.max(1)
    }

    fn measure_optimized_lock_hold(
        state_lock: &Mutex<()>,
        specs: &[EditorJobSpec],
        collections: usize,
    ) -> u128 {
        let mut held_ns = 0_u128;
        let mut checksum = 0_usize;
        for _ in 0..collections {
            let collected = black_box(specs).iter().collect::<Vec<_>>();
            let guard = state_lock.lock().unwrap();
            let started = Instant::now();
            checksum = checksum.wrapping_add(black_box(collected.len()));
            drop(collected);
            held_ns = held_ns.saturating_add(started.elapsed().as_nanos());
            drop(guard);
        }
        black_box(checksum);
        held_ns.max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
