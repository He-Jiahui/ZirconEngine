use crate::core::jobs::{
    test_job_system, CancellationToken, EditorJob, EditorJobSpec, JobCategory, JobContext, JobError,
};

use super::UiHostWindow;

#[test]
fn host_window_uses_only_an_injected_profile_artifact_job_system() {
    let host = UiHostWindow::new().expect("host window should construct");
    assert!(host.profile_artifact_jobs().is_none());

    host.bind_profile_artifact_jobs(test_job_system());

    assert!(host.profile_artifact_jobs().is_some());
}

#[test]
fn profile_artifact_job_is_cancelled_when_the_final_host_owner_drops() {
    let jobs = test_job_system();
    let cancellation = CancellationToken::default();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("Profile artifact cancellation probe", JobCategory::Misc)
                .with_cancel(cancellation.clone()),
            ProfileArtifactCancellationProbe,
        )
        .expect("profile cancellation probe should be admitted");
    let host = UiHostWindow::new().expect("host window should construct");
    host.bind_profile_artifact_jobs(jobs);
    host.track_profile_artifact_job(ticket.id());
    let retained_host = host.clone_strong();

    drop(host);
    assert!(!cancellation.is_cancelled());

    drop(retained_host);
    assert!(matches!(ticket.wait(), Err(JobError::Cancelled)));
}

struct ProfileArtifactCancellationProbe;

impl EditorJob for ProfileArtifactCancellationProbe {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        loop {
            context.check_cancelled()?;
            std::thread::yield_now();
        }
    }
}
