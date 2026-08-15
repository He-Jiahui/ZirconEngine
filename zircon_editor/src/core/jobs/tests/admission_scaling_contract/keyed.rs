use super::support::{GateJob, MergeCancellationJob, ValueJob};
use super::{
    CancellationToken, Duration, EditorJobAdmission, EditorJobAdmissionKey,
    EditorJobAdmissionLimits, EditorJobLimits, EditorJobSpec, JobCategory, JobError, mpsc,
    test_job_system_with_limits,
};

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
fn keyed_pending_merge_refreshes_the_cooperative_cancellation_token() {
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
            EditorJobSpec::new("admission-cancel-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let key = EditorJobAdmissionKey::new("welcome-project:cancel-refresh").unwrap();
    let stale_cancel = CancellationToken::default();
    let accepted = jobs
        .submit_admitted(
            EditorJobSpec::new("admission-cancel-first", JobCategory::Export)
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
    stale_cancel.cancel();

    let current_cancel = CancellationToken::default();
    let merged = jobs
        .submit_admitted(
            EditorJobSpec::new("admission-cancel-latest", JobCategory::Export)
                .with_estimated_bytes(8)
                .with_cancel(current_cancel.clone())
                .with_admission_key(key),
            ValueJob(2),
        )
        .unwrap();
    assert!(matches!(merged, EditorJobAdmission::Merged { .. }));
    assert!(!current_cancel.is_cancelled());

    release_sender.send(()).unwrap();
    assert_eq!(accepted.wait(), Ok(3));
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn cancelling_a_started_merged_job_reaches_the_latest_cancellation_token() {
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
            EditorJobSpec::new("merged-running-cancel-blocker", JobCategory::Export),
            GateJob::new(blocker_started, release_blocker_receiver),
        )
        .unwrap();
    blocker_started_receiver.recv().unwrap();

    let key = EditorJobAdmissionKey::new("welcome-project:running-cancel").unwrap();
    let stale_cancel = CancellationToken::default();
    let (stale_started, _stale_started_receiver) = mpsc::channel();
    let (stale_cancelled, _stale_cancelled_receiver) = mpsc::channel();
    let accepted = jobs
        .submit_admitted(
            EditorJobSpec::new("merged-running-cancel-first", JobCategory::Export)
                .with_estimated_bytes(8)
                .with_cancel(stale_cancel.clone())
                .with_admission_key(key.clone()),
            MergeCancellationJob::new(stale_started, stale_cancelled),
        )
        .unwrap();
    let accepted = match accepted {
        EditorJobAdmission::Accepted(ticket) => ticket,
        EditorJobAdmission::Merged { .. } => panic!("first keyed request must reserve a job"),
    };
    stale_cancel.cancel();

    let current_cancel = CancellationToken::default();
    let (current_started, current_started_receiver) = mpsc::channel();
    let (current_cancelled, current_cancelled_receiver) = mpsc::channel();
    let merged = jobs
        .submit_admitted(
            EditorJobSpec::new("merged-running-cancel-latest", JobCategory::Export)
                .with_estimated_bytes(8)
                .with_cancel(current_cancel.clone())
                .with_admission_key(key),
            MergeCancellationJob::new(current_started, current_cancelled),
        )
        .unwrap();
    assert!(matches!(
        merged,
        EditorJobAdmission::Merged { existing_job } if existing_job == accepted.id()
    ));

    release_blocker.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    current_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("latest merged job must start");
    assert!(jobs.cancel(accepted.id()));
    current_cancelled_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("ticket cancellation must reach the latest merged job");
    assert!(current_cancel.is_cancelled());
    assert_eq!(accepted.wait(), Err(JobError::Cancelled));
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
