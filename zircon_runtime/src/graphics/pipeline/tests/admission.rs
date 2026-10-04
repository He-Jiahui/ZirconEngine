use std::time::Duration;

use super::{PipelineAdmission, PipelineAdmissionReason};

#[test]
fn pipeline_admission_distinguishes_recoverable_defer_from_terminal_failure() {
    let queued = PipelineAdmission::<()>::unavailable(
        PipelineAdmissionReason::CompileQueued,
        Duration::from_micros(7),
    );
    let saturated = PipelineAdmission::<()>::unavailable(
        PipelineAdmissionReason::QueueSaturated,
        Duration::from_micros(11),
    );
    let worker_lost = PipelineAdmission::<()>::unavailable(
        PipelineAdmissionReason::WorkerUnavailable,
        Duration::from_micros(13),
    );

    assert!(queued.is_deferred());
    assert!(saturated.is_deferred());
    assert!(worker_lost.is_failed());
    assert_eq!(
        worker_lost
            .unavailable_details()
            .expect("terminal admission details")
            .state_age(),
        Duration::from_micros(13)
    );
}

#[test]
fn pipeline_admission_reason_labels_are_stable_diagnostic_tokens() {
    assert_eq!(
        PipelineAdmissionReason::CompilePending.label(),
        "compile_pending"
    );
    assert_eq!(
        PipelineAdmissionReason::PipelineValidationFailed.label(),
        "pipeline_validation_failed"
    );
    assert_eq!(
        PipelineAdmissionReason::OitFragmentStoreUnavailable.label(),
        "oit_fragment_store_unavailable"
    );
    assert!(!PipelineAdmissionReason::QueueSaturated.is_terminal());
    assert!(!PipelineAdmissionReason::SourceValidationPending.is_terminal());
    assert!(PipelineAdmissionReason::SourceValidationFailed.is_terminal());
    assert!(PipelineAdmissionReason::ShaderInterfaceMismatch.is_terminal());
    assert!(PipelineAdmissionReason::JobPanicked.is_terminal());
}
