use super::support::{GateJob, ValueJob};
use super::{
    mpsc, test_job_system_with_limits, Duration, EditorJobAdmission, EditorJobAdmissionKey,
    EditorJobAdmissionLimits, EditorJobAdmissionRequest, EditorJobLimits, EditorJobSpec, Instant,
    JobCategory, JobSubmitError,
};

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
            EditorJobSpec::new("admission-entry-blocker", JobCategory::Export)
                .with_estimated_bytes(1),
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
            EditorJobSpec::new("batch-admission-blocker", JobCategory::Export)
                .with_estimated_bytes(1),
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
fn batch_admission_reservation_holds_capacity_until_commit_or_drop() {
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(2, 32, Duration::from_secs(60)),
    ));
    let reservation = jobs
        .reserve_batch_admission(vec![
            EditorJobAdmissionRequest::new(JobCategory::InteractiveSave, 8),
            EditorJobAdmissionRequest::new(JobCategory::InteractiveSave, 8),
        ])
        .unwrap();

    let reserved = jobs.admission_snapshot();
    assert_eq!(reserved.pending_entries(), 2);
    assert_eq!(reserved.pending_estimated_bytes(), 16);
    assert_eq!(
        jobs.submit(
            EditorJobSpec::new("reservation-capacity-race", JobCategory::InteractiveSave),
            ValueJob(1),
        )
        .unwrap_err(),
        JobSubmitError::AdmissionEntryLimitExceeded { limit: 2 }
    );

    let tickets = reservation
        .commit(vec![
            (
                EditorJobSpec::new("reserved-save-a", JobCategory::InteractiveSave)
                    .with_estimated_bytes(8),
                ValueJob(10),
            ),
            (
                EditorJobSpec::new("reserved-save-b", JobCategory::InteractiveSave)
                    .with_estimated_bytes(8),
                ValueJob(20),
            ),
        ])
        .unwrap();
    let mut tickets = tickets.into_iter();
    assert_eq!(tickets.next().unwrap().wait(), Ok(11));
    assert_eq!(tickets.next().unwrap().wait(), Ok(21));

    let released = jobs.admission_snapshot();
    assert_eq!(released.pending_entries(), 0);
    assert_eq!(released.pending_estimated_bytes(), 0);
}

#[test]
fn dropped_batch_admission_reservation_releases_its_reserved_bytes() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            1,
            8,
            Duration::from_secs(60),
        )),
    );
    let reservation = jobs
        .reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
            JobCategory::InteractiveSave,
            8,
        )])
        .unwrap();
    assert_eq!(jobs.admission_snapshot().pending_estimated_bytes(), 8);

    drop(reservation);

    let ticket = jobs
        .submit(
            EditorJobSpec::new("reservation-byte-release", JobCategory::InteractiveSave)
                .with_estimated_bytes(8),
            ValueJob(1),
        )
        .unwrap();
    assert_eq!(ticket.wait(), Ok(2));
}

#[test]
fn shutdown_releases_uncommitted_batch_admission_reservations() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            1,
            8,
            Duration::from_secs(60),
        )),
    );
    let reservation = jobs
        .reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
            JobCategory::InteractiveSave,
            8,
        )])
        .unwrap();

    assert_eq!(jobs.admission_snapshot().pending_entries(), 1);
    assert!(jobs.shutdown(Instant::now()).is_empty());
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);
    assert_eq!(
        reservation
            .commit(vec![(
                EditorJobSpec::new("shutdown-reservation", JobCategory::InteractiveSave)
                    .with_estimated_bytes(8),
                ValueJob(1),
            )])
            .unwrap_err(),
        JobSubmitError::ShuttingDown
    );
}

#[test]
fn shutdown_releases_every_reservation_group_without_leaking_accounting() {
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(4, 16, Duration::from_secs(60)),
    ));
    let first = jobs
        .reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
            JobCategory::InteractiveSave,
            4,
        )])
        .unwrap();
    let second = jobs
        .reserve_batch_admission(vec![EditorJobAdmissionRequest::new(JobCategory::Export, 8)])
        .unwrap();

    assert_eq!(jobs.admission_snapshot().pending_entries(), 2);
    assert_eq!(jobs.admission_snapshot().pending_estimated_bytes(), 12);
    assert!(jobs.shutdown(Instant::now()).is_empty());
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);
    assert_eq!(jobs.admission_snapshot().pending_estimated_bytes(), 0);

    assert_eq!(
        first
            .commit(vec![(
                EditorJobSpec::new("first-shutdown-reservation", JobCategory::InteractiveSave)
                    .with_estimated_bytes(4),
                ValueJob(1),
            ),])
            .unwrap_err(),
        JobSubmitError::ShuttingDown
    );
    assert_eq!(
        second
            .commit(vec![(
                EditorJobSpec::new("second-shutdown-reservation", JobCategory::Export)
                    .with_estimated_bytes(8),
                ValueJob(2),
            ),])
            .unwrap_err(),
        JobSubmitError::ShuttingDown
    );
}

#[test]
fn release_all_reservations_consumes_the_reservation_map_without_id_vec() {
    let source = include_str!("../../system/admission_ledger.rs");
    let release = source
        .split("pub(super) fn release_all_reservations")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn ensure_admissible").next())
        .expect("release-all-reservations implementation");

    assert!(release.contains("std::mem::take(&mut self.reservations)"));
    assert!(release.contains("for (_, reservations) in reservations"));
    assert!(!release.contains("reservation_ids"));
    assert!(!release.contains("self.release_reservation("));
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_release_all_reservations_capacity_evidence() {
    const RESERVATION_GROUPS: usize = 4_096;
    let legacy_id_buffer_allocations = usize::from(RESERVATION_GROUPS > 0);
    let optimized_id_buffer_allocations = 0;

    println!(
        "EDITOR09_RELEASE_ALL_RESERVATIONS_CAPACITY_BENCH_V1 legacy_id_buffer_allocations={} optimized_id_buffer_allocations={} reservation_groups={}",
        legacy_id_buffer_allocations, optimized_id_buffer_allocations, RESERVATION_GROUPS,
    );
    assert_eq!(legacy_id_buffer_allocations, 1);
    assert_eq!(optimized_id_buffer_allocations, 0);
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
            EditorJobSpec::new("admission-byte-blocker", JobCategory::Export)
                .with_estimated_bytes(1),
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
fn category_and_global_snapshots_share_entry_byte_and_lifecycle_counters() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Export, 1)
            .with_limit(JobCategory::Index, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                4,
                8_192,
                Duration::from_secs(60),
            )),
    );
    let (export_started, export_started_receiver) = mpsc::channel();
    let (release_export, release_export_receiver) = mpsc::channel();
    let export_blocker = jobs
        .submit(
            EditorJobSpec::new("category-export-blocker", JobCategory::Export),
            GateJob::new(export_started, release_export_receiver),
        )
        .unwrap();
    export_started_receiver.recv().unwrap();
    let index_blocker = jobs
        .submit(
            EditorJobSpec::new("category-index-blocker", JobCategory::Index)
                .after(export_blocker.id()),
            ValueJob(0),
        )
        .unwrap();
    let index_started_baseline = jobs
        .category_admission_snapshot(JobCategory::Index)
        .started_pending();
    assert_eq!(index_started_baseline, 1);
    let export_pending = jobs
        .submit(
            EditorJobSpec::new("category-export-pending", JobCategory::Export)
                .with_estimated_bytes(32),
            ValueJob(1),
        )
        .unwrap();
    let index_key = EditorJobAdmissionKey::new("category-index:latest").unwrap();
    let index_pending = jobs
        .submit_admitted(
            EditorJobSpec::new("category-index-pending", JobCategory::Index)
                .with_estimated_bytes(4_096)
                .with_admission_key(index_key.clone()),
            ValueJob(1),
        )
        .unwrap();
    let index_pending = match index_pending {
        EditorJobAdmission::Accepted(ticket) => ticket,
        EditorJobAdmission::Merged { .. } => panic!("first category request must be accepted"),
    };
    assert!(matches!(
        jobs.submit_admitted(
            EditorJobSpec::new("category-index-latest", JobCategory::Index)
                .with_estimated_bytes(4_096)
                .with_admission_key(index_key),
            ValueJob(2),
        )
        .unwrap(),
        EditorJobAdmission::Merged { existing_job } if existing_job == index_pending.id()
    ));

    let global = jobs.admission_snapshot();
    let export = jobs.category_admission_snapshot(JobCategory::Export);
    let index = jobs.category_admission_snapshot(JobCategory::Index);
    assert_eq!(global.pending_entries(), 2);
    assert_eq!(global.pending_estimated_bytes(), 32 + 4_096);
    assert_eq!(global.merged_submissions(), 1);
    assert_eq!(export.pending_entries(), 1);
    assert_eq!(export.pending_estimated_bytes(), 32);
    assert_eq!(export.merged_submissions(), 0);
    assert_eq!(index.pending_entries(), 1);
    assert_eq!(index.pending_estimated_bytes(), 4_096);
    assert_eq!(index.merged_submissions(), 1);

    assert!(jobs.cancel(export_pending.id()));
    assert_eq!(
        jobs.category_admission_snapshot(JobCategory::Export)
            .cancelled_pending(),
        1
    );
    assert_eq!(
        jobs.category_admission_snapshot(JobCategory::Index)
            .cancelled_pending(),
        0
    );
    let global_after_cancel = jobs.admission_snapshot();
    let export_after_cancel = jobs.category_admission_snapshot(JobCategory::Export);
    let index_after_cancel = jobs.category_admission_snapshot(JobCategory::Index);
    assert_eq!(
        global_after_cancel.cancelled_pending(),
        export_after_cancel.cancelled_pending() + index_after_cancel.cancelled_pending()
    );

    release_export.send(()).unwrap();
    assert_eq!(export_blocker.wait(), Ok(()));
    assert_eq!(index_blocker.wait(), Ok(1));
    assert_eq!(index_pending.wait(), Ok(3));
    assert_eq!(
        jobs.category_admission_snapshot(JobCategory::Index)
            .started_pending(),
        index_started_baseline + 1
    );
    let global_after_start = jobs.admission_snapshot();
    let export_after_start = jobs.category_admission_snapshot(JobCategory::Export);
    let index_after_start = jobs.category_admission_snapshot(JobCategory::Index);
    assert_eq!(
        global_after_start.started_pending(),
        export_after_start.started_pending() + index_after_start.started_pending()
    );
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
            EditorJobSpec::new("admission-age-blocker", JobCategory::Export)
                .with_estimated_bytes(1),
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
