use super::*;
use zr_rhi::{DeviceGeneration, DeviceId, RenderQueueClass};

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(3),
        DeviceGeneration::new(2),
        RenderQueueClass::Graphics,
        sequence,
    )
}

fn submissions() -> Vec<RenderFrameSubmissionProducerRecord> {
    vec![RenderFrameSubmissionProducerRecord::new(
        RenderFrameSubmissionProducer::FrameResourceUpload,
        ticket(39),
    )]
}

fn poll() -> SubmissionPollReceipt {
    SubmissionPollReceipt::new(DeviceId::new(3), DeviceGeneration::new(2), 11)
}

#[test]
fn failure_receipt_retains_submitted_pre_scene_identity() {
    let receipt = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions(),
        vec![SubmissionStatus::Submitted],
        None,
    )
    .expect("failure receipt");

    assert_eq!(receipt.frame_generation(), 7);
    assert_eq!(receipt.pre_scene_submissions().len(), 1);
    assert_eq!(receipt.pre_scene_submissions()[0].ticket(), ticket(39));
    assert_eq!(
        receipt.pre_scene_submissions()[0].status(),
        SubmissionStatus::Submitted
    );
}

#[test]
fn failure_receipt_retains_typed_physical_boundary_reason() {
    let texture_id = ResourceId::from_stable_label("failure-texture");
    let submissions = vec![RenderFrameSubmissionProducerRecord::for_resource_boundary(
        RenderFrameSubmissionProducer::TexturePreUpload,
        texture_id,
        RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload,
        ticket(39),
    )];
    let receipt = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions,
        vec![SubmissionStatus::Submitted],
        None,
    )
    .expect("failure receipt");

    assert_eq!(
        receipt.pre_scene_submissions()[0].boundary_reason(),
        Some(RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload)
    );
}

#[test]
fn failure_receipt_rejects_unsettled_accepted_submission() {
    let error = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions(),
        vec![SubmissionStatus::Accepted],
        None,
    )
    .expect_err("accepted work is not a failure terminal disposition");

    assert!(matches!(
        error,
        RenderFrameSubmissionFailureReceiptError::SubmissionRemainedAccepted { .. }
    ));
}

#[test]
fn failure_receipt_rejects_incomplete_status_ledger() {
    let error = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions(),
        Vec::new(),
        None,
    )
    .expect_err("every submission needs a settlement status");

    assert!(matches!(
        error,
        RenderFrameSubmissionFailureReceiptError::StatusCountMismatch { .. }
    ));
}

#[test]
fn failure_receipt_retains_scene_ticket_when_finalization_fails_after_submit() {
    let receipt = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions(),
        vec![SubmissionStatus::Submitted],
        Some(ticket(40)),
    )
    .expect("submitted scene failure receipt");

    assert_eq!(receipt.scene_submission(), Some(ticket(40)));
}

#[test]
fn failure_receipt_rejects_foreign_submitted_scene_owner() {
    let foreign_scene = SubmissionTicket::new(
        DeviceId::new(4),
        DeviceGeneration::new(2),
        RenderQueueClass::Graphics,
        40,
    );
    let error = RenderFrameSubmissionFailureReceipt::from_transaction(
        7,
        poll(),
        submissions(),
        vec![SubmissionStatus::Submitted],
        Some(foreign_scene),
    )
    .expect_err("submitted scene must use the frame device generation");

    assert!(matches!(
        error,
        RenderFrameSubmissionFailureReceiptError::SceneOwnerMismatch { .. }
    ));
}
