use super::*;

#[test]
fn pipeline_compile_rejects_duplicate_stage_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline.renderer.stages.push(RenderPassStage::Opaque3d);

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_feature_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_core_pipeline_mismatch_from_extract() {
    let error = RenderPipelineAsset::default_forward_plus()
        .compile(&orthographic_extract())
        .unwrap_err();

    assert!(
        error.contains("core pipeline mismatch")
            && error.contains("Core3d")
            && error.contains("Core2d"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_declared_renderer_stage_missing_product_phase_mapping() {
    let mut pipeline = RenderPipelineAsset::default_core2d();
    pipeline
        .phase_mapping
        .retain(|phase| *phase != RenderPhase::Transparent2d);

    let error = pipeline.compile(&orthographic_extract()).unwrap_err();

    assert!(
        error.contains("missing product phase") && error.contains("Transparent2d"),
        "unexpected error: {error}"
    );
}

#[test]
fn disabling_post_process_keeps_overlay_extract_requirements_for_debug_gizmos() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("post_process"),
        )
        .unwrap();
    let pass_names = compiled
        .graph()
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"uber"),
        "uber pass should be removed when the feature is disabled"
    );
    assert!(
        !pass_names.contains(&"depth-of-field-prepare"),
        "DoF scratch preparation should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-depth-pyramid"),
        "screen-space reflection depth pyramid should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-reflection-pyramid"),
        "screen-space reflection reflection pyramid should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-depth-pyramid-coarse"),
        "screen-space reflection coarse depth pyramid should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-reflection-pyramid-coarse"),
        "screen-space reflection coarse reflection pyramid should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-resolve"),
        "screen-space reflection resolve should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"screen-space-reflection-specular-occlusion"),
        "screen-space reflection specular occlusion should be removed with the post-process feature"
    );
    assert!(
        pass_names.contains(&"velocity-object"),
        "object velocity producer belongs to the temporal feature, not post-process"
    );
    assert!(
        pass_names.contains(&"velocity-camera"),
        "camera velocity producer belongs to the temporal feature, not post-process"
    );
    assert!(
        !pass_names.contains(&"motion-vector-tile-max"),
        "motion-vector tile reconstruction should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"motion-vector-tile-max-coarse"),
        "coarse motion-vector tile reconstruction should be removed with the post-process feature"
    );
    assert!(
        !pass_names.contains(&"motion-vector-neighbor-max"),
        "motion-vector reconstruction should be removed with the post-process feature"
    );
    assert!(
        pass_names.contains(&"overlay-gizmo"),
        "overlay stage should remain available for debug and gizmo rendering"
    );
    assert!(
        compiled
            .required_extract_sections
            .contains(&"debug".to_string()),
        "overlay feature should keep requiring debug extract data"
    );
}

#[test]
fn effective_post_process_stack_culls_disabled_optional_post_process_passes() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        false,
        false,
        &Default::default(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();
    let live_pass_names = compiled
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(live_pass_names.contains(&"uber"));
    for pass_name in [
        "velocity-object",
        "velocity-camera",
        "taa-reactive-mask-mesh",
        "taa-resolve",
        "motion-vector-tile-max",
        "motion-vector-tile-max-coarse",
        "motion-vector-neighbor-max",
        "depth-of-field-prepare",
        "screen-space-reflection-reflection-pyramid",
        "screen-space-reflection-reflection-pyramid-coarse",
        "screen-space-reflection-specular-occlusion",
        "screen-space-reflection-resolve",
        "upscale",
    ] {
        assert!(
            !live_pass_names.contains(&pass_name),
            "`{pass_name}` should be culled when the effective post-process stack does not request it; live={live_pass_names:?}"
        );
    }

    let lifetimes = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .map(|lifetime| lifetime.name.as_str())
        .collect::<Vec<_>>();
    for resource_name in [
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
        PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
        PostProcessGraphResourceNames::TAA_OUTPUT,
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        PostProcessGraphResourceNames::UPSCALED,
    ] {
        assert!(
            !lifetimes.contains(&resource_name),
            "`{resource_name}` should not keep a graph lifetime when its effect family is disabled; lifetimes={lifetimes:?}"
        );
    }
}

#[test]
fn effective_post_process_stack_keeps_screen_space_reflection_passes_when_requested() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 32,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &Default::default(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();
    let live_pass_names = compiled
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    for pass_name in [
        "velocity-object",
        "velocity-camera",
        "motion-vector-tile-max",
        "motion-vector-neighbor-max",
        "hzb-occlusion-cull",
        "hzb-build",
        "screen-space-reflection-reflection-pyramid",
        "screen-space-reflection-reflection-pyramid-coarse",
        "screen-space-reflection-specular-occlusion",
        "screen-space-reflection-resolve",
    ] {
        assert!(
            live_pass_names.contains(&pass_name),
            "`{pass_name}` should remain live when SSR requests it; live={live_pass_names:?}"
        );
    }
}

#[test]
fn renderer_feature_asset_quality_gate_controls_compiled_passes() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom")
        .quality_gate = Some(BuiltinRenderFeature::RayTracing);

    let without_gate = pipeline.compile(&test_extract()).unwrap();
    assert!(!without_gate
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));

    let with_gate = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::RayTracing),
        )
        .unwrap();
    assert!(with_gate
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
}

#[test]
fn pipeline_compile_validates_quality_gated_descriptor_overrides_even_when_gate_is_closed() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque,
                "bad-gated-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("bad-gated-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn renderer_feature_asset_local_config_and_capabilities_survive_compile() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let color_grading = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("default pipeline should include color grading");
    *color_grading = color_grading
        .clone()
        .with_local_config("variant", "cinematic")
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline);

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let compiled_color_grading = compiled
        .enabled_features()
        .iter()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("color grading should remain enabled");

    assert_eq!(
        compiled_color_grading.local_config.get("variant"),
        Some(&"cinematic".to_string())
    );
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::RayTracingPipeline));
}

#[test]
fn renderer_feature_asset_descriptor_override_changes_compiled_graph() {
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
            "custom-bloom",
            vec!["custom_post".to_string()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "custom-bloom-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .read_texture("scene-color")
            .write_external("viewport-output")],
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph()
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"bloom-extract"));
    assert!(pass_names.contains(&"custom-bloom-pass"));
    assert!(compiled
        .required_extract_sections
        .contains(&"custom_post".to_string()));
    assert!(compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .any(|lifetime| {
            lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
        }));
}
