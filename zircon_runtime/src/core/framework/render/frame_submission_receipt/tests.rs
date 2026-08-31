use super::*;
use zr_rhi::RenderQueueClass;

fn ticket(device: u64, generation: u64, sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(device),
        DeviceGeneration::new(generation),
        RenderQueueClass::Graphics,
        sequence,
    )
}

fn poll(device: u64, generation: u64, sequence: u64) -> SubmissionPollReceipt {
    SubmissionPollReceipt::new(
        DeviceId::new(device),
        DeviceGeneration::new(generation),
        sequence,
    )
}

#[test]
fn render_frame_submission_receipt_keeps_one_device_generation_timeline() {
    let receipt = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_present_submission(ticket(3, 2, 41))
        .expect("present must advance the same timeline");

    assert_eq!(receipt.frame_generation(), 7);
    assert_eq!(receipt.poll().sequence(), 11);
    assert_eq!(receipt.scene_submission().sequence(), 40);
    assert_eq!(
        receipt.present_submission().map(|ticket| ticket.sequence()),
        Some(41)
    );
}

#[test]
fn render_frame_submission_receipt_rejects_foreign_poll_owner() {
    let error = RenderFrameSubmissionReceipt::new(7, poll(4, 2, 11), ticket(3, 2, 40))
        .expect_err("foreign poll owner must fail closed");

    assert!(matches!(
        error,
        RenderFrameSubmissionReceiptError::PollOwnerMismatch { .. }
    ));
}

#[test]
fn render_frame_submission_receipt_rejects_present_before_scene() {
    let error = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_present_submission(ticket(3, 2, 39))
        .expect_err("present cannot precede scene submission");

    assert_eq!(
        error,
        RenderFrameSubmissionReceiptError::PresentPrecededScene {
            scene_sequence: 40,
            present_sequence: 39,
        }
    );
}

#[test]
fn render_frame_submission_receipt_accepts_present_in_the_scene_packet() {
    let receipt = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_viewport_product_submission(ticket(3, 2, 40))
        .expect("product copy shares the scene packet")
        .with_present_submission(ticket(3, 2, 40))
        .expect("surface blit shares the scene packet");

    assert_eq!(receipt.present_submission(), Some(ticket(3, 2, 40)));
}

#[test]
fn render_frame_submission_receipt_rejects_foreign_present_owner() {
    let error = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_present_submission(ticket(4, 2, 41))
        .expect_err("present must stay on the scene device generation");

    assert!(matches!(
        error,
        RenderFrameSubmissionReceiptError::PresentOwnerMismatch { .. }
    ));
}

#[test]
fn render_frame_submission_receipt_orders_viewport_product_before_present() {
    let receipt = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_viewport_product_submission(ticket(3, 2, 41))
        .expect("product copy must follow scene")
        .with_present_submission(ticket(3, 2, 42))
        .expect("present must follow product publication");

    assert_eq!(
        receipt
            .viewport_product_submission()
            .map(|submission| submission.sequence()),
        Some(41)
    );
    assert_eq!(
        receipt
            .present_submission()
            .map(|submission| submission.sequence()),
        Some(42)
    );
}

#[test]
fn render_frame_submission_receipt_accepts_product_copy_in_the_scene_packet() {
    let receipt = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_viewport_product_submission(ticket(3, 2, 40))
        .expect("scene-tail product copy shares the scene ticket");

    assert_eq!(
        receipt.viewport_product_submission(),
        Some(ticket(3, 2, 40))
    );
}

#[test]
fn render_frame_submission_receipt_rejects_product_submission_before_scene() {
    let error = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_viewport_product_submission(ticket(3, 2, 39))
        .expect_err("product copy cannot precede scene submission");

    assert_eq!(
        error,
        RenderFrameSubmissionReceiptError::ViewportProductPrecededScene {
            scene_sequence: 40,
            viewport_product_sequence: 39,
        }
    );
}

#[test]
fn render_frame_submission_receipt_requires_publication_to_match_recorded_product() {
    let receipt = RenderFrameSubmissionReceipt::new(7, poll(3, 2, 11), ticket(3, 2, 40))
        .expect("matching frame identity")
        .with_viewport_product_submission(ticket(3, 2, 40))
        .expect("scene-tail product copy shares the scene ticket");

    receipt
        .validate_viewport_product_publication(7, ticket(3, 2, 40))
        .expect("publication must accept the recorded scene-tail copy");
    assert_eq!(
        receipt
            .validate_viewport_product_publication(7, ticket(3, 2, 41))
            .expect_err("publication must reject a different product ticket"),
        RenderFrameSubmissionReceiptError::ViewportProductIdentityMismatch {
            recorded: Some(ticket(3, 2, 40)),
            published: ticket(3, 2, 41),
        }
    );
    assert_eq!(
        receipt
            .validate_viewport_product_publication(8, ticket(3, 2, 40))
            .expect_err("publication must reject a different frame generation"),
        RenderFrameSubmissionReceiptError::ViewportProductFrameGenerationMismatch {
            frame_generation: 7,
            viewport_product_generation: 8,
        }
    );
}

#[test]
fn render_frame_submission_receipt_keeps_interval_metrics_separate_from_shared_products() {
    let texture_id = ResourceId::from_stable_label("receipt-texture");
    let pre_scene_submissions =
        std::sync::Arc::from([RenderFrameSubmissionProducerRecord::for_resource(
            RenderFrameSubmissionProducer::TextureCopyUpload,
            texture_id,
            ticket(3, 2, 39),
        )]);
    let metrics = RenderFrameSubmissionMetrics::new(2, 2, 1, 0, 1, 0, 1, 2048);
    let receipt = RenderFrameSubmissionReceipt::from_transaction(
        7,
        poll(3, 2, 11),
        ticket(3, 2, 40),
        Some(pre_scene_submissions),
    )
    .expect("matching frame identity")
    .with_submission_metrics(Some(metrics))
    .with_viewport_product_submission(ticket(3, 2, 40))
    .expect("product copy shares the scene packet")
    .with_present_submission(ticket(3, 2, 40))
    .expect("present shares the scene packet");

    assert_eq!(receipt.logical_packet_count(), 2);
    assert_eq!(receipt.submission_metrics(), Some(metrics));
    assert_eq!(
        receipt
            .submission_metrics()
            .map(RenderFrameSubmissionMetrics::physical_backend_submission_count),
        Some(1)
    );
}
