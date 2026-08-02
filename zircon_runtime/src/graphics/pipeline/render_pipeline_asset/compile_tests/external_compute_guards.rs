use super::*;

#[test]
fn compile_preserves_required_external_texture_binding() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(80),
        revision: 1,
        name: "required-external-texture-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "required-external-texture-renderer".to_string(),
            stages: vec![RenderPassStage::PostProcess],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "required-external-texture-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "history-consumer",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("test.history-consumer")
                    .with_side_effects()
                    .read_required_external_texture("history.previous-color")],
                ),
            )],
        },
    };

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let lifetime = compiled
        .graph()
        .resource_lifetime_by_name("history.previous-color")
        .expect("required external texture lifetime");

    assert_eq!(
        lifetime.external_binding,
        RenderGraphExternalResourceBinding::required_texture()
    );
}

#[test]
fn compile_rejects_conflicting_required_external_texture_and_buffer_binding() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(81),
        revision: 1,
        name: "required-external-conflict-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "required-external-conflict-renderer".to_string(),
            stages: vec![RenderPassStage::PostProcess],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "required-external-conflict-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "conflicting-external",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("test.conflicting-external")
                    .with_side_effects()
                    .read_required_external_texture("shared.external")
                    .write_required_external_buffer("shared.external")],
                ),
            )],
        },
    };

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `shared.external` has conflicting external resource binding"),
        "{error}"
    );
}

#[test]
fn compile_rejects_compute_workload_on_non_compute_queue() {
    let pipeline =
        RenderPipelineAsset {
            handle: RenderPipelineHandle::new(79),
            revision: 1,
            name: "compute-workload-queue-test".to_string(),
            core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
            phase_mapping: vec![RenderPhase::PostProcess],
            renderer: RendererAsset {
                name: "compute-workload-queue-renderer".to_string(),
                stages: vec![RenderPassStage::PostProcess],
                features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                    RenderFeatureDescriptor::new(
                        "invalid-compute-workload-feature",
                        Vec::new(),
                        Vec::new(),
                        vec![RenderFeaturePassDescriptor::new(
                            RenderPassStage::PostProcess,
                            "bad-compute",
                            QueueLane::Graphics,
                        )
                        .with_executor_id("bad.compute")
                        .with_compute_workload(
                            RenderGraphComputeWorkload::fixed("bad-pipeline", [1, 1, 1], [1, 1, 1]),
                        )],
                    ),
                )],
            },
        };

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains(
            "feature descriptor `invalid-compute-workload-feature` pass `bad-compute` cannot declare compute workload on `Graphics` queue"
        ),
        "{error}"
    );
}
