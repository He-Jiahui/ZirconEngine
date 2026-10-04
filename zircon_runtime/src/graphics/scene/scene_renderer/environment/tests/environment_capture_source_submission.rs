use super::EnvironmentCaptureSourceSubmissionStatus;

const SOURCE: &str = include_str!("../environment_capture_source_submission.rs");

#[test]
fn source_submission_retains_target_and_both_backend_tickets() {
    for field in [
        "handle: RenderEnvironmentCaptureHandle",
        "target: EnvironmentCaptureGpuTarget",
        "resource_upload_submission: SubmissionTicket",
        "capture_submission: SubmissionTicket",
        "record_report: EnvironmentCaptureWgpuRecordReport",
        "filter_report: EnvironmentCaptureFilterWgpuRecordReport",
        "probe_publication: Option<EnvironmentCaptureProbePublication>",
    ] {
        assert!(
            SOURCE.contains(field),
            "missing source owner field: {field}"
        );
    }
}

#[test]
fn submission_status_requires_both_backend_tickets_to_complete() {
    use zr_rhi::SubmissionStatus::{Completed, DeviceLost, Submitted};

    assert_eq!(
        EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, Completed),
        EnvironmentCaptureSourceSubmissionStatus::Completed
    );
    assert_eq!(
        EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, Submitted),
        EnvironmentCaptureSourceSubmissionStatus::Pending
    );
    assert!(matches!(
        EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, DeviceLost),
        EnvironmentCaptureSourceSubmissionStatus::Failed { .. }
    ));
}
