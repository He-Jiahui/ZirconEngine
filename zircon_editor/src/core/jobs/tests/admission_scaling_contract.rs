use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use super::super::{
    EditorJob, EditorJobAdmission, EditorJobAdmissionKey, EditorJobAdmissionLimits,
    EditorJobLimits, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, JobSubmitError,
    test_job_system_with_limits,
};

const MAX_BUCKET_PROBES_PER_PASS: usize = JobPriority::ALL.len() * JobCategory::ALL.len();

#[test]
fn indexed_pending_admission_scales_linearly_through_enqueue_and_completion() {
    let small = admission_probe_sample(1_000);
    let large = admission_probe_sample(10_000);

    assert!(
        small <= (1_000 * 2 + 2) * MAX_BUCKET_PROBES_PER_PASS,
        "unexpected 1k enqueue+completion probe count: {small}"
    );
    assert!(
        large <= (10_000 * 2 + 2) * MAX_BUCKET_PROBES_PER_PASS,
        "unexpected 10k enqueue+completion probe count: {large}"
    );
    assert!(
        large <= small.saturating_mul(11),
        "10k enqueue+completion promotion grew faster than linear: 1k={small}, 10k={large}"
    );
}

#[test]
fn ready_bucket_selection_cannot_regress_to_a_linear_job_scan() {
    let source = include_str!("../system/pending.rs");
    let take_next = source
        .split("pub(super) fn take_next")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn mark_dependency_schedulable")
                .next()
        })
        .expect("take_next source section");

    assert!(source.contains("BTreeSet<JobId>"));
    assert!(take_next.contains("ids.first().copied()"));
    for retired_scan in [".iter()", ".position(", ".min_by_key(", ".remove(index)"] {
        assert!(
            !take_next.contains(retired_scan),
            "ready selection restored a linear pending scan: {retired_scan}"
        );
    }
}

#[test]
fn promotion_uses_a_bounded_dispatch_batch_without_holding_the_state_mutex_for_schedule() {
    let source = include_str!("../system/mod.rs");
    let promote = source
        .split("fn promote(self: &Arc<Self>)")
        .nth(1)
        .and_then(|body| body.split("fn finish").next())
        .expect("promotion body should remain available");

    assert!(promote.contains("MAX_PROMOTION_DISPATCH_BATCH"));
    assert!(promote.contains("let _promotion ="));
    let state_selection = promote
        .find("let dispatch = {")
        .expect("pending selection must be scoped under the state mutex");
    let schedule = promote
        .find("let handle = self.scheduler.schedule_after")
        .expect("selected work must schedule through the runtime");
    assert!(state_selection < schedule);
    assert!(
        promote[state_selection..schedule].contains("let (pending, dependencies) = dispatch"),
        "runtime scheduling must begin after the state-lock selection scope ends"
    );
}

#[test]
fn admission_bucket_inventory_has_one_enum_owned_source() {
    let pending_source = include_str!("../system/pending.rs");
    let category_source = include_str!("../category.rs");

    assert!(pending_source.contains("JobPriority::ALL"));
    assert!(pending_source.contains("JobCategory::ALL"));
    assert!(!pending_source.contains("const PRIORITIES"));
    assert!(!pending_source.contains("const CATEGORIES"));
    assert_eq!(category_source.matches("define_job_enum! {").count(), 2);
    assert!(category_source.contains("[Self; [$(stringify!($variant)),+].len()]"));
    assert!(category_source.contains("[$(Self::$variant),+]"));
    assert!(!category_source.contains("pub const ALL: [Self; 3]"));
    assert!(!category_source.contains("pub const ALL: [Self; 8]"));
    assert_eq!(JobPriority::ALL.len(), 3);
    assert_eq!(JobCategory::ALL.len(), 8);
}

#[test]
fn pending_admission_rejects_entry_overflow_and_releases_cancelled_capacity() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                1,
                32,
                Duration::from_secs(60),
            )),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission-entry-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let pending = jobs
        .submit(
            EditorJobSpec::new("admission-entry-pending", JobCategory::Export)
                .with_estimated_bytes(8),
            ValueJob(1),
        )
        .unwrap();
    let snapshot = jobs.admission_snapshot();
    assert_eq!(snapshot.pending_entries(), 1);
    assert_eq!(snapshot.pending_estimated_bytes(), 8);
    assert!(snapshot.oldest_pending_age().is_some());

    assert_eq!(
        jobs.submit(
            EditorJobSpec::new("admission-entry-overflow", JobCategory::Export)
                .with_estimated_bytes(1),
            ValueJob(2),
        )
        .unwrap_err(),
        JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 }
    );

    assert!(jobs.cancel(pending.id()));
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);
    let retry = jobs
        .submit(
            EditorJobSpec::new("admission-entry-retry", JobCategory::Export)
                .with_estimated_bytes(8),
            ValueJob(3),
        )
        .unwrap();
    assert!(jobs.cancel(retry.id()));
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn batch_admission_rejects_atomically_without_retaining_partial_tickets() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                1,
                32,
                Duration::from_secs(60),
            )),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("batch-admission-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    assert_eq!(
        jobs.submit_batch(vec![
            (
                EditorJobSpec::new("batch-first", JobCategory::Export).with_estimated_bytes(8),
                ValueJob(1),
            ),
            (
                EditorJobSpec::new("batch-second", JobCategory::Export).with_estimated_bytes(8),
                ValueJob(2),
            ),
        ])
        .unwrap_err(),
        JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 }
    );
    let snapshot = jobs.admission_snapshot();
    assert_eq!(snapshot.pending_entries(), 0);
    assert_eq!(snapshot.pending_estimated_bytes(), 0);

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn pending_admission_rejects_declared_byte_overflow() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(4, 8, Duration::from_secs(60))),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission-byte-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let pending = jobs
        .submit(
            EditorJobSpec::new("admission-byte-pending", JobCategory::Export)
                .with_estimated_bytes(8),
            ValueJob(1),
        )
        .unwrap();
    assert_eq!(
        jobs.submit(
            EditorJobSpec::new("admission-byte-overflow", JobCategory::Export)
                .with_estimated_bytes(1),
            ValueJob(2),
        )
        .unwrap_err(),
        JobSubmitError::AdmissionByteLimitExceeded {
            limit: 8,
            current: 8,
            requested: 1,
        }
    );

    assert!(jobs.cancel(pending.id()));
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn pending_admission_rejects_when_the_oldest_wait_exceeds_its_budget() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(4, 32, Duration::ZERO)),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission-age-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let pending = jobs
        .submit(
            EditorJobSpec::new("admission-age-pending", JobCategory::Export)
                .with_estimated_bytes(8),
            ValueJob(1),
        )
        .unwrap();
    assert_eq!(
        jobs.submit(
            EditorJobSpec::new("admission-age-overflow", JobCategory::Export)
                .with_estimated_bytes(1),
            ValueJob(2),
        )
        .unwrap_err(),
        JobSubmitError::OldestPendingAgeExceeded { max_age_ms: 0 }
    );

    assert!(jobs.cancel(pending.id()));
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn keyed_pending_admission_returns_merged_without_consuming_another_reservation() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                4,
                32,
                Duration::from_secs(60),
            )),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission-key-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let key = EditorJobAdmissionKey::new("welcome-project:current").unwrap();
    let accepted = jobs
        .submit_admitted(
            EditorJobSpec::new("admission-key-first", JobCategory::Export)
                .with_estimated_bytes(8)
                .with_admission_key(key.clone()),
            ValueJob(1),
        )
        .unwrap();
    let accepted = match accepted {
        EditorJobAdmission::Accepted(ticket) => ticket,
        EditorJobAdmission::Merged { .. } => panic!("first keyed request must reserve a job"),
    };
    let accepted_id = accepted.id();

    let merged = jobs
        .submit_admitted(
            EditorJobSpec::new("admission-key-latest", JobCategory::Export)
                .with_estimated_bytes(8)
                .with_admission_key(key),
            ValueJob(2),
        )
        .unwrap();
    assert!(matches!(
        merged,
        EditorJobAdmission::Merged { existing_job } if existing_job == accepted_id
    ));
    let snapshot = jobs.admission_snapshot();
    assert_eq!(snapshot.pending_entries(), 1);
    assert_eq!(snapshot.pending_estimated_bytes(), 8);
    assert_eq!(snapshot.merged_submissions(), 1);

    release_sender.send(()).unwrap();
    assert_eq!(
        accepted.wait(),
        Ok(3),
        "merged payload must replace stale work"
    );
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn keyed_admission_applies_the_request_wait_age_before_allocating_a_new_job() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                4,
                32,
                Duration::from_secs(60),
            )),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission-age-request-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("admission-age-request-pending", JobCategory::Export)
                .with_estimated_bytes(8),
            ValueJob(1),
        )
        .unwrap();

    assert_eq!(
        jobs.submit_admitted(
            EditorJobSpec::new("admission-age-request-overflow", JobCategory::Export)
                .with_admission_key(EditorJobAdmissionKey::new("save:current").unwrap())
                .with_max_pending_age(Duration::ZERO),
            ValueJob(2),
        )
        .unwrap_err(),
        JobSubmitError::OldestPendingAgeExceeded { max_age_ms: 0 }
    );

    assert!(jobs.cancel(pending.id()));
    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

fn admission_probe_sample(job_count: usize) -> usize {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("probe-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let baseline = jobs.admission_probe_count();

    let tickets = (0..job_count)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("probe-{index}"), JobCategory::Export)
                    .with_priority(JobPriority::Background),
                ValueJob(index as u32),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    for (index, ticket) in tickets.into_iter().enumerate() {
        assert_eq!(ticket.wait(), Ok(index as u32 + 1));
    }
    assert_eq!(jobs.pending_job_count(), 0);
    assert_eq!(jobs.running_job_count(), 0);

    jobs.admission_probe_count().saturating_sub(baseline)
}

struct ValueJob(u32);

impl EditorJob for ValueJob {
    type Output = u32;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(self.0 + 1)
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
        let _ = self.started.send(());
        self.release.recv().map_err(JobError::failed)
    }
}
