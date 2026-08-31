use super::*;

#[test]
fn pipeline_compile_rejects_descriptor_passes_for_undeclared_stages() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-stage-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Opaque,
                    "custom-gbuffer-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .with_side_effects(),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("custom-gbuffer-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_descriptor_pass_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "uber",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .with_side_effects(),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate render graph pass name `uber`"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_conflicting_descriptor_resource_kinds() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "bad-resource-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .write_buffer("scene-color"),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("conflicting resource kind"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_explicit_external_resource_name_conflicts() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-external-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "bad-external-resource-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .write_external("scene-color"),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("explicit external resource"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "",
                    QueueLane::Graphics,
                )
                .with_executor_id("")
                .write_texture(""),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("feature descriptor name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_names_after_descriptor_name_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber"),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("pass name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "empty-resource-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("")
                .write_texture(""),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("executor id must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_resource_names_after_executor_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "empty-resource-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .write_texture(""),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_storage_write_mode_on_read_access() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    let mut invalid_pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "bad-storage-mode-pass",
        QueueLane::Graphics,
    )
    .with_executor_id("post.uber");
    invalid_pass
        .resources
        .push(RenderFeatureResourceDescriptor {
            name: "scene-color".to_string(),
            kind: RenderFeatureResourceKind::Texture,
            access: RenderFeatureResourceAccess::Read,
            input_version: None,
            minimum_size_bytes: None,
            attachment_ops: None,
            write_mode: RenderFeatureResourceWriteMode::Storage,
            access_metadata: None,
            external_binding: crate::render_graph::RenderGraphExternalResourceBinding::report_only(
            ),
            texture_view_alias: None,
            schema: None,
            usage: crate::render_graph::RenderGraphResourceUsageFlags::default(),
        });
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-storage-mode-feature",
            Vec::new(),
            Vec::new(),
            vec![invalid_pass],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("cannot declare storage write mode for a read access"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_extract_section_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-extract-section-feature",
            vec!["post".to_string(), " ".to_string()],
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "post-stack-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .read_texture("scene-color"),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("extract section name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_history_bindings_in_one_descriptor() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-history-feature",
            Vec::new(),
            vec![
                FrameHistoryBinding::read(FrameHistorySlot::TaaSceneColor),
                FrameHistoryBinding::write(FrameHistorySlot::TaaSceneColor),
            ],
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "history-using-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber")
                .read_texture("scene-color"),
            ],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate history binding for slot `TaaSceneColor`"),
        "unexpected error: {error}"
    );
}
