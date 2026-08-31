use super::{assert_contains_all, assert_not_contains};

#[test]
fn runtime09a_frame_submission_metrics_keep_backend_mapping_and_receipt_owners_split() {
    let boundary_reason =
        include_str!("../../../../core/framework/render/frame_submission_boundary_reason.rs");
    let producer_record =
        include_str!("../../../../core/framework/render/frame_submission_producer_record.rs");
    let metrics = include_str!("../../../../core/framework/render/frame_submission_metrics.rs");
    let receipt = include_str!("../../../../core/framework/render/frame_submission_receipt.rs");
    let receipt_tests =
        include_str!("../../../../core/framework/render/frame_submission_receipt/tests.rs");
    let transaction =
        include_str!("../../../../core/framework/render/frame_submission_transaction.rs");
    let transaction_tests =
        include_str!("../../../../core/framework/render/frame_submission_transaction/tests.rs");
    let backend =
        include_str!("../../../../graphics/backend/render_backend/render_backend_submission.rs");
    let direct_frame = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs"
    );
    let compiled_frame = include_str!(
        "../../../../graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs"
    );

    assert!(boundary_reason.contains("TextureMipPreservationBeforeUpload"));
    assert!(!boundary_reason.contains("wgpu::"));
    assert_contains_all(
        "frame submission producer record",
        producer_record,
        &[
            "pub struct RenderFrameSubmissionProducerRecord",
            "boundary_reason: Option<RenderFrameSubmissionBoundaryReason>",
            "mismatched_boundary_reason",
        ],
    );
    assert_contains_all(
        "backend-neutral frame submission metrics",
        metrics,
        &[
            "pub struct RenderFrameSubmissionMetrics",
            "admitted_logical_packet_count",
            "flushed_logical_ticket_count",
            "physical_backend_submission_count",
            "upload_payload_bytes",
        ],
    );
    assert_not_contains(
        "backend-neutral frame submission metrics",
        metrics,
        &["wgpu::", "WgpuSubmissionMetricsSnapshot"],
    );
    assert_contains_all(
        "frame submission receipt metrics attachment",
        receipt,
        &[
            "submission_metrics: Option<RenderFrameSubmissionMetrics>",
            "BoundaryReasonProducerMismatch",
            "mismatched_boundary_reason",
            "pub fn logical_packet_count(&self) -> u64",
            "pub(crate) fn with_submission_metrics(",
            "#[path = \"frame_submission_receipt/tests.rs\"]",
        ],
    );
    assert_contains_all(
        "frame submission receipt tests owner",
        receipt_tests,
        &[
            "keeps_interval_metrics_separate_from_shared_products",
            "physical_backend_submission_count",
        ],
    );
    assert_contains_all(
        "frame submission transaction boundary validation",
        transaction,
        &[
            "record_pre_scene_resource_submission_with_boundary",
            "mismatched_boundary_reason",
            "BoundaryReasonProducerMismatch",
            "#[path = \"frame_submission_transaction/tests.rs\"]",
        ],
    );
    assert!(
        transaction_tests.contains("transaction_rejects_boundary_reason_for_the_wrong_producer")
    );
    assert_contains_all(
        "WGPU frame submission metrics mapper",
        backend,
        &[
            "pub(crate) fn frame_submission_metrics_since(",
            "record_pre_scene_resource_submission_with_boundary(",
            "self.submission_metrics().delta_since(baseline)?",
            "RenderFrameSubmissionMetrics::new(",
        ],
    );
    for (label, frame_owner) in [
        ("direct frame owner", direct_frame),
        ("compiled frame owner", compiled_frame),
    ] {
        assert_contains_all(
            label,
            frame_owner,
            &[
                "let submission_metrics_baseline = self.backend.submission_metrics();",
                ".frame_submission_metrics_since(",
                ".with_submission_metrics(",
            ],
        );
    }

    assert!(boundary_reason.lines().count() < 80);
    assert!(producer_record.lines().count() < 120);
    assert!(metrics.lines().count() < 120);
    assert!(receipt.lines().count() < 400);
    assert!(receipt_tests.lines().count() < 250);
    assert!(transaction.lines().count() < 220);
    assert!(transaction_tests.lines().count() < 280);
    assert!(backend.lines().count() < 400);
    assert!(direct_frame.lines().count() < 500);
    assert!(compiled_frame.lines().count() < 450);
}
