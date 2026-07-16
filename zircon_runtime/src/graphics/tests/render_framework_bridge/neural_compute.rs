use super::*;

#[test]
fn render_framework_rejects_neural_compute_plugin_descriptor_without_executor_registration() {
    let server = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [neural_compute_render_feature_descriptor()],
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    let mut pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([neural_compute_render_feature_descriptor()]);
    pipeline.handle = RenderPipelineHandle::new(86);
    pipeline.name = "neural-descriptor-only-pipeline".to_string();

    let error = server.register_pipeline_asset(pipeline).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::GraphCompileFailure {
            pipeline: 86,
            message:
                "render pass `plugin-neural-inference` references unregistered executor `plugin.neural.inference`"
                    .to_string(),
        }
    );
}

#[test]
fn render_framework_rejects_neural_compute_plugin_pipeline_when_backend_capability_is_missing() {
    let server = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [neural_compute_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            "plugin.neural.inference",
            neural_compute_render_pass_executor,
        )],
        Vec::new(),
    )
    .unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());
    let mut pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([neural_compute_render_feature_descriptor()]);
    pipeline.handle = RenderPipelineHandle::new(87);
    pipeline.name = "neural-capability-gated-pipeline".to_string();
    let handle = server.register_pipeline_asset(pipeline).unwrap();

    let error = server.set_pipeline_asset(viewport, handle).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 87,
            reason: "pipeline `neural-capability-gated-pipeline` requires neural_compute"
                .to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::NeuralCompute]),
        }
    );
}
