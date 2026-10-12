use super::*;

use crate::core::jobs::JobSubmitError;
use crate::ui::retained_host::host_contract::profiling_artifacts::ProfileOutputRootError;

#[test]
fn rejected_profile_artifact_submission_records_a_host_warning() {
    let host = UiHostWindow::new().expect("host window should construct");

    assert_eq!(
        profile_artifact_submission_job_id(
            &host,
            Err(ProfileArtifactSubmissionError::Job(
                JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 },
            )),
        ),
        None
    );

    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Warning
    );
    assert!(diagnostics[0]
        .message()
        .contains("profile artifact export was not submitted"));
}

#[test]
fn invalid_profile_output_root_records_a_host_warning() {
    let host = UiHostWindow::new().expect("host window should construct");

    assert_eq!(
        profile_artifact_submission_job_id(
            &host,
            Err(ProfileArtifactSubmissionError::InvalidOutputRoot(
                ProfileOutputRootError,
            )),
        ),
        None
    );

    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Warning
    );
    assert!(diagnostics[0]
        .message()
        .contains("outside the C: system drive"));
}
