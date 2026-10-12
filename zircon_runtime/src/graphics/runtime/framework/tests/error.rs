use crate::core::framework::render::{
    RenderFrameSubmissionReceipt, RenderFrameworkError, RenderPipelinePhase,
    RenderSceneSubmissionCompletionError,
};
use crate::graphics::GraphicsError;
use zr_rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
};

use super::render_framework_backend_error;

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(5),
        DeviceGeneration::new(3),
        RenderQueueClass::Graphics,
        sequence,
    )
}

#[test]
fn product_publication_error_keeps_scene_and_copy_identity_at_framework_boundary() {
    let receipt = RenderFrameSubmissionReceipt::new(
        9,
        SubmissionPollReceipt::new(DeviceId::new(5), DeviceGeneration::new(3), 7),
        ticket(40),
    )
    .expect("scene receipt");
    let error = render_framework_backend_error(GraphicsError::FrameProductPublicationFailed {
        receipt: receipt.clone(),
        product_submission: Some(ticket(41)),
        source: Box::new(GraphicsError::SurfaceStatus("copy validation")),
    });

    assert_eq!(error.frame_submission_receipt(), Some(&receipt));
    assert_eq!(error.product_submission(), Some(ticket(41)));
}

#[test]
fn missing_view_family_phase_remains_typed_at_framework_boundary() {
    let error = render_framework_backend_error(GraphicsError::MissingViewFamilyPhase {
        phase: RenderPipelinePhase::SceneLinear,
    });

    assert_eq!(
        error,
        RenderFrameworkError::MissingViewFamilyPhase {
            phase: RenderPipelinePhase::SceneLinear,
        }
    );
}

#[test]
fn compiled_scene_preparation_errors_remain_typed_at_framework_boundary() {
    assert_eq!(
        render_framework_backend_error(GraphicsError::MissingFrameGraphResourceBacking {
            resource: "scene-velocity",
        }),
        RenderFrameworkError::MissingFrameGraphResourceBacking {
            resource: "scene-velocity",
        }
    );
    assert_eq!(
        render_framework_backend_error(GraphicsError::MissingPreparedGpuSceneUpload),
        RenderFrameworkError::MissingPreparedGpuSceneUpload
    );
    assert_eq!(
        render_framework_backend_error(GraphicsError::InvalidBufferUploadRange {
            label: "scene-uniform",
        }),
        RenderFrameworkError::InvalidBufferUploadRange {
            label: "scene-uniform",
        }
    );
}

#[test]
fn rejected_producer_settlement_remains_typed_at_framework_boundary() {
    let error = render_framework_backend_error(GraphicsError::FrameProducerRegistrationFailed {
        ticket: ticket(39),
        status: zr_rhi::SubmissionStatus::Cancelled,
        source: Box::new(GraphicsError::SurfaceStatus("ledger validation")),
    });

    assert_eq!(
        error,
        RenderFrameworkError::FrameProducerRegistrationFailed {
            ticket: ticket(39),
            status: zr_rhi::SubmissionStatus::Cancelled,
            reason: "wgpu surface acquisition status: ledger validation".to_string(),
        }
    );
}

#[test]
fn scene_completion_error_remains_typed_at_framework_boundary() {
    let error = RenderSceneSubmissionCompletionError::PollSequenceDidNotAdvance {
        previous_sequence: 11,
        poll_sequence: 11,
    };

    assert_eq!(
        render_framework_backend_error(GraphicsError::SceneSubmissionCompletion(error)),
        RenderFrameworkError::SceneSubmissionCompletion(error),
    );
}
