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

fn transaction() -> RenderFrameSubmissionTransaction {
    RenderFrameSubmissionTransaction::begin(
        7,
        SubmissionPollReceipt::new(DeviceId::new(3), DeviceGeneration::new(2), 11),
    )
}

#[test]
fn empty_transaction_finishes_without_a_pre_scene_allocation() {
    let receipt = transaction().finish(ticket(40)).expect("scene receipt");

    assert!(receipt.pre_scene_submissions().is_empty());
}

#[test]
fn transaction_retains_ordered_pre_scene_submission_identity() {
    let mut transaction = transaction();
    transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(39),
        )
        .expect("history ticket");
    let receipt = transaction.finish(ticket(40)).expect("scene receipt");

    assert_eq!(receipt.pre_scene_submissions().len(), 1);
    assert_eq!(
        receipt.pre_scene_submissions()[0].producer(),
        RenderFrameSubmissionProducer::FrameResourceUpload
    );
    assert_eq!(receipt.pre_scene_submissions()[0].ticket(), ticket(39));
}

#[test]
fn transaction_rejects_non_monotonic_producer_tickets() {
    let mut transaction = transaction();
    transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(39),
        )
        .expect("first producer");
    let error = transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(38),
        )
        .expect_err("producer sequence must advance");

    assert!(matches!(
        error,
        RenderFrameSubmissionReceiptError::ProducerSequenceDidNotAdvance { .. }
    ));
}

#[test]
fn transaction_rejects_foreign_producer_owner() {
    let mut transaction = transaction();
    let foreign = SubmissionTicket::new(
        DeviceId::new(4),
        DeviceGeneration::new(2),
        RenderQueueClass::Graphics,
        39,
    );
    let error = transaction
        .record_pre_scene_submission(RenderFrameSubmissionProducer::FrameResourceUpload, foreign)
        .expect_err("producer must use the frame device generation");

    assert!(matches!(
        error,
        RenderFrameSubmissionReceiptError::ProducerOwnerMismatch { .. }
    ));
}

#[test]
fn transaction_rejects_producer_after_scene_sequence() {
    let mut transaction = transaction();
    transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(41),
        )
        .expect("producer owner");
    let error = transaction
        .finish(ticket(40))
        .expect_err("producer must precede scene");

    assert!(matches!(
        error,
        RenderFrameSubmissionReceiptError::ProducerDidNotPrecedeScene { .. }
    ));
}

#[test]
fn transaction_abort_preserves_ticket_order_and_settled_statuses() {
    let mut transaction = transaction();
    transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(39),
        )
        .expect("history ticket");

    assert_eq!(transaction.pre_scene_submission_tickets(), vec![ticket(39)]);
    let receipt = transaction
        .abort(vec![SubmissionStatus::Submitted])
        .expect("settled failure receipt");

    assert_eq!(receipt.pre_scene_submissions().len(), 1);
    assert_eq!(
        receipt.pre_scene_submissions()[0].status(),
        SubmissionStatus::Submitted
    );
}

#[test]
fn transaction_retains_texture_pre_copy_post_order() {
    let mut transaction = transaction();
    let texture_id = ResourceId::from_stable_label("transaction-texture");
    for (producer, sequence, boundary_reason) in [
        (
            RenderFrameSubmissionProducer::TexturePreUpload,
            36,
            Some(RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload),
        ),
        (RenderFrameSubmissionProducer::TextureCopyUpload, 37, None),
        (RenderFrameSubmissionProducer::TexturePostUpload, 38, None),
    ] {
        match boundary_reason {
            Some(boundary_reason) => transaction
                .record_pre_scene_resource_submission_with_boundary(
                    producer,
                    texture_id,
                    boundary_reason,
                    ticket(sequence),
                )
                .expect("ordered texture boundary producer"),
            None => transaction
                .record_pre_scene_resource_submission(producer, texture_id, ticket(sequence))
                .expect("ordered texture producer"),
        }
    }

    let receipt = transaction.finish(ticket(40)).expect("scene receipt");
    let producers = receipt
        .pre_scene_submissions()
        .iter()
        .map(|record| record.producer())
        .collect::<Vec<_>>();

    assert_eq!(
        producers,
        vec![
            RenderFrameSubmissionProducer::TexturePreUpload,
            RenderFrameSubmissionProducer::TextureCopyUpload,
            RenderFrameSubmissionProducer::TexturePostUpload,
        ]
    );
    assert!(receipt
        .pre_scene_submissions()
        .iter()
        .all(|record| record.resource_id() == Some(texture_id)));
    assert_eq!(
        receipt
            .pre_scene_submissions()
            .iter()
            .map(|record| record.boundary_reason())
            .collect::<Vec<_>>(),
        vec![
            Some(RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload),
            None,
            None,
        ]
    );
}

#[test]
fn transaction_rejects_boundary_reason_for_the_wrong_producer() {
    let mut transaction = transaction();
    let error = transaction
        .record_pre_scene_resource_submission_with_boundary(
            RenderFrameSubmissionProducer::TextureCopyUpload,
            ResourceId::from_stable_label("invalid-boundary-texture"),
            RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload,
            ticket(36),
        )
        .expect_err("copy upload cannot claim the pre-upload preservation boundary");

    assert_eq!(
        error,
        RenderFrameSubmissionReceiptError::BoundaryReasonProducerMismatch {
            producer: RenderFrameSubmissionProducer::TextureCopyUpload,
            boundary_reason:
                RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload,
        }
    );
    assert!(transaction.pre_scene_submission_tickets().is_empty());
}

#[test]
fn transaction_abort_after_scene_retains_the_submitted_scene_ticket() {
    let mut transaction = transaction();
    transaction
        .record_pre_scene_submission(
            RenderFrameSubmissionProducer::FrameResourceUpload,
            ticket(39),
        )
        .expect("history submission");

    let receipt = transaction
        .abort_after_scene_submission(ticket(40), vec![SubmissionStatus::Submitted])
        .expect("submitted scene failure receipt");

    assert_eq!(receipt.scene_submission(), Some(ticket(40)));
}
