use super::*;

#[test]
fn rendering_plugin_default_features_restore_legacy_forward_plus_pass_order() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
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
            "preview-sky",
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
        .graph()
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
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
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
            "preview-sky",
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

#[test]
fn rendering_plugin_default_features_preserve_motion_vector_and_bloom_composite_contract() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline
            .with_plugin_render_features(default_rendering_feature_descriptors())
            .compile(&test_extract())
            .unwrap();
        let pass_names = compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>();
        let pass_index = |name| {
            pass_names
                .iter()
                .position(|pass| *pass == name)
                .unwrap_or_else(|| panic!("plugin default graph should keep `{name}`"))
        };

        pass_resource_access(
            &compiled,
            "motion-vector-tile-max",
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphResourceAccessKind::Read,
        );
        assert!(
            pass_index("bloom-extract") < pass_index("reflection-probe-composite")
                && pass_index("reflection-probe-composite")
                    < pass_index("baked-lighting-composite")
                && pass_index("baked-lighting-composite") < pass_index("motion-vector-tile-max"),
            "the default plugin graph must sample Bloom before scene-color composites and begin the motion-vector chain afterwards"
        );
    }
}

#[test]
fn plugin_feature_buffer_minimum_size_survives_graph_resource_planning() {
    const PLUGIN_PACKET_SIZE_BYTES: u64 = 1_280;

    let descriptor = RenderFeatureDescriptor::new(
        "fixed-size-plugin-packet",
        vec!["view".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "fixed-size-plugin-packet-write",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("test.fixed-size-plugin-packet")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "test-fixed-size-plugin-packet",
                [1, 1, 1],
                [1, 1, 1],
            ))
            .with_side_effects()
            .write_buffer_with_minimum_size("fixed-size-plugin-packet", PLUGIN_PACKET_SIZE_BYTES),
        ],
    );

    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(vec![descriptor])
        .compile(&test_extract())
        .unwrap();
    let lifetime = graph_resource_lifetime(&compiled, "fixed-size-plugin-packet");
    let RenderGraphResourceDesc::Buffer(desc) = &lifetime.desc else {
        panic!("fixed-size plugin packet should compile as a transient buffer");
    };

    assert_eq!(desc.size_bytes, PLUGIN_PACKET_SIZE_BYTES);
}
