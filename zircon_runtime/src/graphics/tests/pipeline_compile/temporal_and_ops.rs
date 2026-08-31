use super::*;

#[test]
fn taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        true,
        true,
        &AntiAliasSettings::taa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::Temporal)
                .with_post_process_stack(stack),
        )
        .unwrap();
    let live_pass_names = compiled
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(live_pass_names.contains(&"taa-resolve"));
    assert!(live_pass_names.contains(&"taa-reactive-mask-mesh"));
    assert!(live_pass_names.contains(&"velocity-object"));
    assert!(live_pass_names.contains(&"velocity-camera"));
    assert!(
        compiled
            .history_bindings
            .contains(&FrameHistoryBinding::read_write(
                FrameHistorySlot::TaaSceneColor
            )),
        "TAA resolve must declare the scene-color history slot; bindings={:?}",
        compiled.history_bindings
    );
    for pass_name in [
        "motion-vector-tile-max",
        "motion-vector-tile-max-coarse",
        "motion-vector-neighbor-max",
    ] {
        assert!(
            !live_pass_names.contains(&pass_name),
            "`{pass_name}` should stay culled for TAA-only scene velocity; live={live_pass_names:?}"
        );
    }

    let taa_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-resolve")
        .expect("TAA resolve pass should be compiled when TAA is effective");
    assert_eq!(
        taa_pass.executor_id.as_deref(),
        Some("temporal.taa-resolve")
    );
    let reactive_mask_mesh_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-reactive-mask-mesh")
        .expect("TAA reactive mask mesh pass should be compiled when TAA is effective");
    assert_eq!(
        reactive_mask_mesh_pass.executor_id.as_deref(),
        Some("temporal.taa-reactive-mask-mesh")
    );
    pass_resource_access(
        &compiled,
        "taa-reactive-mask-mesh",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    let reactive_mask_mesh_write = pass_resource_access(
        &compiled,
        "taa-reactive-mask-mesh",
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        reactive_mask_mesh_write.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        reactive_mask_mesh_write.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "taa-resolve",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "taa-resolve",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "taa-resolve",
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "taa-resolve",
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "taa-resolve",
            PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
            RenderGraphResourceAccessKind::Read,
        )
        .kind,
        RenderGraphResourceKind::External
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "taa-resolve",
            PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            RenderGraphResourceAccessKind::Write,
        )
        .kind,
        RenderGraphResourceKind::External
    );
    let taa_output_write = pass_resource_access(
        &compiled,
        "taa-resolve",
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        taa_output_write.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        taa_output_write.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    );

    let taa_output = graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::TAA_OUTPUT);
    assert!(matches!(
        &taa_output.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.format == TextureFormat::Rg11b10Ufloat && desc.sample_count == 1
    ));
    let reactive_mask =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::TAA_REACTIVE_MASK);
    assert!(matches!(
        &reactive_mask.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.format == TextureFormat::R8Unorm && desc.sample_count == 1
    ));
}

#[test]
fn taa_resolve_reads_depth_of_field_output_from_pre_reconstruction_phase() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::taa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::Temporal)
                .with_post_process_stack(stack),
        )
        .unwrap();

    let taa_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-resolve")
        .expect("TAA resolve should be compiled");
    let taa_read_resources = taa_pass
        .resources
        .iter()
        .filter(|resource| resource.access == RenderGraphResourceAccessKind::Read)
        .map(|resource| resource.name.as_str())
        .collect::<Vec<_>>();
    assert!(taa_read_resources.contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED));
    assert!(!taa_read_resources.contains(&PostProcessGraphResourceNames::SCENE_COLOR));

    let live_pass_names = compiled
        .execution_passes_in_graph_order()
        .map(|execution_pass| {
            compiled.graph().passes()[execution_pass.graph_pass_index]
                .name
                .as_str()
        })
        .collect::<Vec<_>>();
    let depth_of_field_index = live_pass_names
        .iter()
        .position(|name| *name == "depth-of-field")
        .expect("depth of field should remain live");
    let taa_index = live_pass_names
        .iter()
        .position(|name| *name == "taa-resolve")
        .expect("TAA resolve should remain live");
    assert!(depth_of_field_index < taa_index);
}

#[test]
fn taa_resolve_pass_and_resources_are_absent_when_taa_is_disabled() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        true,
        true,
        &AntiAliasSettings::off(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::Temporal)
                .with_post_process_stack(stack),
        )
        .unwrap();
    let live_pass_names = compiled
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    let lifetimes = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .map(|lifetime| lifetime.name.as_str())
        .collect::<Vec<_>>();

    assert!(!live_pass_names.contains(&"taa-resolve"));
    assert!(
        !compiled
            .history_bindings
            .contains(&FrameHistoryBinding::read_write(
                FrameHistorySlot::TaaSceneColor
            )),
        "TAA-disabled temporal compile should not reserve scene-color history; bindings={:?}",
        compiled.history_bindings
    );
    for resource_name in [
        PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
        PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
        PostProcessGraphResourceNames::TAA_OUTPUT,
    ] {
        assert!(
            !lifetimes.contains(&resource_name),
            "`{resource_name}` should not be allocated when TAA is disabled; lifetimes={lifetimes:?}"
        );
    }
}

#[test]
fn pipeline_compile_assigns_attachment_ops_from_resource_write_order() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    let preview_sky_depth = pass_resource_access(
        &compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(preview_sky_depth.attachment_ops, None);

    let preview_sky_scene_color = pass_resource_access(
        &compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "preview sky should preserve opaque color before filling far-depth background"
    );

    let prepass_depth = pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        prepass_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "depth prepass should own the first graph depth clear"
    );

    let prepass_normal = pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        prepass_normal.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "depth prepass should clear the graph-owned normal target before writing normals"
    );

    let opaque_scene_color = pass_resource_access(
        &compiled,
        "opaque-mesh",
        "scene-color",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        opaque_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "opaque scene color should declare the graph's first write; the camera-stack policy converts it to a load after the frame-level clear"
    );

    let transparent_scene_color = pass_resource_access(
        &compiled,
        "transparent-mesh",
        "scene-color",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        transparent_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "later scene-color producers must load existing opaque output"
    );

    let runtime_ui_output = pass_resource_access(
        &compiled,
        "runtime-ui",
        "viewport-output",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        runtime_ui_output.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "runtime UI must load the overlay/postprocess output before the frame tail write"
    );

    let overlay_output = pass_resource_access(
        &compiled,
        "overlay-gizmo",
        "viewport-output",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        overlay_output.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "overlay must load the postprocess output before adding debug draws"
    );

    let deferred_compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();
    let deferred_lighting_scene_color = pass_resource_access(
        &deferred_compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        deferred_lighting_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred lighting should declare the graph's first scene-color write before sky composition"
    );
    let deferred_preview_sky_scene_color = pass_resource_access(
        &deferred_compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        deferred_preview_sky_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred preview sky should preserve lit geometry before filling the background"
    );

    let deferred_prepass_depth = pass_resource_access(
        &deferred_compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        deferred_prepass_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred depth prepass should own the first graph depth clear"
    );
}
