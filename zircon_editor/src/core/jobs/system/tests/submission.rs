use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use crate::core::jobs::{
    test_job_system_with_limits, CancellationToken, EditorJob, EditorJobAdmission,
    EditorJobAdmissionKey, EditorJobAdmissionLimits, EditorJobLimits, EditorJobSpec, JobCategory,
    JobContext, JobError,
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
            EditorJobSpec::new("merge-cancel-blocker", JobCategory::Export).with_estimated_bytes(1),
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
