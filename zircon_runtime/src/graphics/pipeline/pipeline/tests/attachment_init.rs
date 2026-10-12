use crate::graphics::feature::RenderFeaturePassDescriptor;
use crate::render_graph::QueueLane;

use super::*;

#[test]
fn normalizes_sequential_attachment_writes_to_clear_then_load() {
    let mut descriptors = vec![RenderFeatureDescriptor::new(
        "attachment-initialization",
        Vec::new(),
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "attachment-seed",
                QueueLane::Graphics,
            )
            .with_executor_id("test.attachment-seed")
            .write_texture("attachment-color"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "attachment-compose",
                QueueLane::Graphics,
            )
            .with_executor_id("test.attachment-compose")
            .write_texture("attachment-color"),
        ],
    )];

    normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
        .expect("sequential attachment writes normalize");

    let seed = &descriptors[0].stage_passes[0].resources[0];
    let compose = &descriptors[0].stage_passes[1].resources[0];
    assert_eq!(
        seed.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert_eq!(
        compose.attachment_ops,
        Some(RenderGraphAttachmentOps::load_store())
    );
    assert_eq!(
        compose
            .input_version
            .as_ref()
            .map(|version| version.producer_pass_name()),
        Some("attachment-seed")
    );
}

#[test]
fn preserves_external_attachment_initial_contents_as_load() {
    let mut descriptors = vec![RenderFeatureDescriptor::new(
        "external-attachment-initialization",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "external-compose",
            QueueLane::Graphics,
        )
        .with_executor_id("test.external-compose")
        .write_external_texture("external-color")],
    )];

    normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
        .expect("external attachment writes normalize");

    let attachment = &descriptors[0].stage_passes[0].resources[0];
    assert_eq!(
        attachment.attachment_ops,
        Some(RenderGraphAttachmentOps::load_store())
    );
    assert!(attachment.input_version.is_none());
}

#[test]
fn rejects_a_producer_token_without_an_attachment_load_operation() {
    let mut descriptors = vec![RenderFeatureDescriptor::new(
        "attachment-token-without-load",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "attachment-token-consumer",
            QueueLane::Graphics,
        )
        .with_executor_id("test.attachment-token-consumer")
        .write_texture("attachment-color")],
    )];
    descriptors[0].stage_passes[0].resources[0].input_version =
        Some(RenderFeatureResourceVersion::new(
            "attachment-color",
            RenderFeatureResourceKind::Texture,
            "attachment-seed",
        ));

    let error =
        normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
            .expect_err("a producer token requires an attachment load operation");

    assert!(error.contains("without a Load operation"), "{error}");
}
