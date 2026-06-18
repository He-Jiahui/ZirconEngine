use crate::core::framework::render::{RenderFrameExtract, RenderPhase, RenderWorldSnapshotHandle};
use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
use crate::graphics::pipeline::{RenderPassStage, RenderPipelineAsset, RendererAsset};
use crate::render_graph::{QueueLane, RenderGraphExternalResourceBinding, RenderGraphResourceKind};
use crate::scene::world::World;

#[test]
fn compile_preserves_report_only_external_texture_binding() {
    let compiled = optional_external_pipeline(
        RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "optional-texture-consumer",
            QueueLane::Graphics,
        )
        .with_executor_id("test.optional-texture-consumer")
        .with_side_effects()
        .read_external_texture("history.optional-color"),
    )
    .compile(&test_extract())
    .unwrap();

    let lifetime = compiled
        .graph
        .resource_lifetime_by_name("history.optional-color")
        .expect("optional external texture lifetime");

    assert_eq!(lifetime.kind, RenderGraphResourceKind::External);
    assert_eq!(
        lifetime.external_binding,
        RenderGraphExternalResourceBinding::report_only_texture()
    );
}

#[test]
fn compile_preserves_report_only_external_buffer_binding() {
    let compiled = optional_external_pipeline(
        RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "optional-buffer-consumer",
            QueueLane::Graphics,
        )
        .with_executor_id("test.optional-buffer-consumer")
        .with_side_effects()
        .read_external_buffer("history.optional-exposure"),
    )
    .compile(&test_extract())
    .unwrap();

    let lifetime = compiled
        .graph
        .resource_lifetime_by_name("history.optional-exposure")
        .expect("optional external buffer lifetime");

    assert_eq!(lifetime.kind, RenderGraphResourceKind::External);
    assert_eq!(
        lifetime.external_binding,
        RenderGraphExternalResourceBinding::report_only_buffer()
    );
}

#[test]
fn compile_rejects_conflicting_report_only_external_texture_and_buffer_binding() {
    let pipeline = optional_external_pipeline(
        RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "optional-conflict",
            QueueLane::Graphics,
        )
        .with_executor_id("test.optional-conflict")
        .with_side_effects()
        .read_external_texture("history.shared")
        .read_external_buffer("history.shared"),
    );

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `history.shared` has conflicting external resource binding"),
        "{error}"
    );
}

fn optional_external_pipeline(pass: RenderFeaturePassDescriptor) -> RenderPipelineAsset {
    RenderPipelineAsset {
        handle: crate::core::framework::render::RenderPipelineHandle::new(82),
        revision: 1,
        name: "optional-external-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "optional-external-renderer".to_string(),
            stages: vec![RenderPassStage::PostProcess],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "optional-external-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![pass],
                ),
            )],
        },
    }
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
