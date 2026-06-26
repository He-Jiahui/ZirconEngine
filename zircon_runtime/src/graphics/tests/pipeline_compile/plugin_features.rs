use super::*;

#[test]
fn rendering_plugin_default_features_restore_legacy_forward_plus_pass_order() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "hzb-occlusion-cull",
            "velocity-object",
            "velocity-camera",
            "shadow-atlas",
            "hzb-build",
            "ssao-evaluate",
            "light-grid-build",
            "opaque-mesh",
            "alpha-mask-mesh",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "motion-vector-tile-max",
            "motion-vector-tile-max-coarse",
            "motion-vector-neighbor-max",
            "depth-of-field-prepare",
            "screen-space-reflection-reflection-pyramid",
            "screen-space-reflection-reflection-pyramid-coarse",
            "screen-space-reflection-specular-occlusion",
            "screen-space-reflection-resolve",
            "uber",
            "output-transfer",
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::LIGHT_LIST,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::TONEMAPPED,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "output-transfer",
        PostProcessGraphResourceNames::TONEMAPPED,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "output-transfer",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        compiled.history_bindings,
        vec![
            FrameHistoryBinding::read_write(FrameHistorySlot::AmbientOcclusion),
            FrameHistoryBinding::read_write(FrameHistorySlot::HzbFurthest)
        ]
    );
}

#[test]
fn rendering_plugin_post_process_routes_output_transfer_through_terminal_anti_alias_input() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_anti_alias(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        false,
        false,
        &AntiAliasSettings::fxaa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors())
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "output-transfer",
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "fxaa",
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "fxaa",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    let output_transfer = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("plugin post-process should keep output transfer");
    assert!(!output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
}

#[test]
fn rendering_plugin_default_features_restore_legacy_deferred_pass_order() {
    let pipeline = RenderPipelineAsset::default_deferred()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "hzb-occlusion-cull",
            "velocity-object",
            "velocity-camera",
            "shadow-atlas",
            "gbuffer-mesh",
            "hzb-build",
            "ssao-evaluate",
            "light-grid-build",
            "deferred-lighting",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "motion-vector-tile-max",
            "motion-vector-tile-max-coarse",
            "motion-vector-neighbor-max",
            "depth-of-field-prepare",
            "screen-space-reflection-reflection-pyramid",
            "screen-space-reflection-reflection-pyramid-coarse",
            "screen-space-reflection-specular-occlusion",
            "screen-space-reflection-resolve",
            "uber",
            "output-transfer",
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
    );
}
