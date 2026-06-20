use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, PreviewEnvironmentExtract, ProjectionMode, RenderCameraTarget,
    RenderDynamicResolutionSettings, RenderFrameExtract, RenderPhase,
    RenderPostProcessEffectStackSettings, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderScreenSpaceReflectionSettings, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphComputeWorkload, RenderGraphResourceAccessKind, RenderGraphResourceDesc,
    RenderGraphResourceKind,
};
use crate::rhi::TextureFormat;

use crate::graphics::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};

#[test]
fn default_forward_plus_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert!(pipeline.phase_mapping.contains(&RenderPhase::AlphaMask3d));

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque3d,
            RenderPassStage::AlphaMask3d,
            RenderPassStage::Transparent3d,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
            RenderPassStage::Ui,
        ]
    );
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
            "light-grid-build",
            "opaque-mesh",
            "alpha-mask-mesh",
            "transparent-mesh",
            "motion-vector-tile-max",
            "motion-vector-tile-max-coarse",
            "motion-vector-neighbor-max",
            "depth-of-field-prepare",
            "depth-of-field",
            "motion-blur",
            "exposure-histogram",
            "exposure-resolve",
            "screen-space-reflection-reflection-pyramid",
            "screen-space-reflection-reflection-pyramid-coarse",
            "screen-space-reflection-specular-occlusion",
            "screen-space-reflection-resolve",
            "scene-composite",
            "blur",
            "bloom-extract",
            "uber",
            "output-transfer",
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "velocity-object",
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "velocity-object",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "depth-of-field-prepare",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "depth-of-field-prepare",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "depth-of-field-prepare",
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "depth-of-field-prepare",
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "hzb-occlusion-cull",
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-reflection-pyramid",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "screen-space-reflection-reflection-pyramid",
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-reflection-pyramid-coarse",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "screen-space-reflection-reflection-pyramid-coarse",
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "velocity-camera",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "velocity-camera",
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::load_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-tile-max",
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-tile-max",
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-tile-max-coarse",
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-tile-max-coarse",
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-neighbor-max",
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-neighbor-max",
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-specular-occlusion",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-specular-occlusion",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-specular-occlusion",
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "screen-space-reflection-specular-occlusion",
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        RenderGraphResourceAccessKind::Read,
    );
    let screen_space_reflection_history_write = pass_resource_access(
        &compiled,
        "screen-space-reflection-resolve",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        screen_space_reflection_history_write.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        screen_space_reflection_history_write.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
        RenderGraphResourceAccessKind::Read,
    );
    let screen_space_reflection_history_read = pass_resource_access(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        screen_space_reflection_history_read.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::HzbFurthest
        )]
    );
}

#[test]
fn default_deferred_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_deferred();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert!(pipeline.phase_mapping.contains(&RenderPhase::AlphaMask3d));

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::Deferred,
            RenderPassStage::AlphaMask3d,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Transparent3d,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
            RenderPassStage::Ui,
        ]
    );
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
            "light-grid-build",
            "deferred-lighting",
            "transparent-mesh",
            "motion-vector-tile-max",
            "motion-vector-tile-max-coarse",
            "motion-vector-neighbor-max",
            "depth-of-field-prepare",
            "depth-of-field",
            "motion-blur",
            "exposure-histogram",
            "exposure-resolve",
            "screen-space-reflection-reflection-pyramid",
            "screen-space-reflection-reflection-pyramid-coarse",
            "screen-space-reflection-specular-occlusion",
            "screen-space-reflection-resolve",
            "scene-composite",
            "blur",
            "bloom-extract",
            "uber",
            "output-transfer",
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "velocity-object",
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "velocity-object",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "velocity-camera",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "velocity-camera",
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::load_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-tile-max",
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-tile-max",
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-tile-max-coarse",
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-tile-max-coarse",
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "motion-vector-neighbor-max",
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "motion-vector-neighbor-max",
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    pass_resource_access(
        &compiled,
        "hzb-occlusion-cull",
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "gbuffer-mesh",
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "gbuffer-mesh",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    assert!(
        compiled
            .graph
            .passes()
            .iter()
            .flat_map(|pass| pass.resources.iter())
            .any(|resource| resource.name == PostProcessGraphResourceNames::GBUFFER_MATERIAL),
        "default deferred graph should declare its backed material G-buffer"
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::HzbFurthest
        )]
    );
}

#[test]
fn deferred_material_gbuffer_shaders_encode_and_decode_material_channels() {
    let geometry_shader =
        include_str!("../scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl");
    let lighting_shader =
        include_str!("../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl");
    for (name, shader) in [
        ("deferred_geometry.wgsl", geometry_shader),
        ("deferred_lighting.wgsl", lighting_shader),
    ] {
        let module = naga::front::wgsl::parse_str(shader)
            .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(shader)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    assert!(
        geometry_shader.contains("@location(1) material: vec4<f32>"),
        "deferred geometry should emit a second material G-buffer target"
    );
    assert!(
        geometry_shader.contains("material_properties.data0.x")
            && geometry_shader.contains("material_properties.data0.y"),
        "deferred geometry should encode metallic and roughness from material uniform channels"
    );
    assert!(
        lighting_shader.contains("var gbuffer_material_tex: texture_2d<f32>")
            && lighting_shader.contains("textureLoad(gbuffer_material_tex"),
        "deferred lighting should read the material G-buffer"
    );
    assert!(
        lighting_shader.contains("let roughness =") && lighting_shader.contains("let metallic ="),
        "deferred lighting should decode material G-buffer channels"
    );
}

#[test]
fn dynamic_resolution_scales_internal_graph_resources_without_resizing_viewport_output() {
    let mut extract = test_extract();
    extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    extract.apply_viewport_size(UVec2::new(320, 240));

    assert_eq!(extract.view.effective_view_size(), UVec2::new(320, 240));
    assert_eq!(extract.view.effective_render_size(), UVec2::new(160, 120));

    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        extract.post_process.exposure,
        &extract.post_process.effect_stack,
        false,
        false,
        &AntiAliasSettings::off(),
        true,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let scene_color =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::SCENE_COLOR);
    assert!(matches!(
        &scene_color.desc,
        RenderGraphResourceDesc::Texture(desc)
            if scene_color.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 160
                && desc.height == 120
                && desc.format == TextureFormat::Rgba8UnormSrgb
    ));

    let scene_depth =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::SCENE_DEPTH);
    assert!(matches!(
        &scene_depth.desc,
        RenderGraphResourceDesc::Texture(desc)
            if scene_depth.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 160
                && desc.height == 120
        && desc.format == TextureFormat::Depth32Float
    ));

    let upscaled = graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::UPSCALED);
    assert!(matches!(
        &upscaled.desc,
        RenderGraphResourceDesc::Texture(desc)
            if upscaled.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 320
                && desc.height == 240
                && desc.format == TextureFormat::Rgba8Unorm
    ));

    let upscale_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "upscale")
        .expect("dynamic resolution should compile the explicit upscale pass");
    assert!(upscale_pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::TONEMAPPED
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(upscale_pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::UPSCALED
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let output_transfer = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("dynamic resolution should still compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::UPSCALED
            && resource.access == RenderGraphResourceAccessKind::Read
    }));

    let viewport_output = graph_resource_lifetime(&compiled, "viewport-output");
    assert_eq!(viewport_output.kind, RenderGraphResourceKind::External);
    assert_eq!(viewport_output.desc, RenderGraphResourceDesc::External);
}

#[test]
fn dynamic_resolution_keeps_terminal_anti_alias_input_at_viewport_size() {
    let mut extract = test_extract();
    extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    extract.apply_viewport_size(UVec2::new(320, 240));

    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        extract.post_process.exposure,
        &extract.post_process.effect_stack,
        false,
        false,
        &AntiAliasSettings::fxaa(),
        true,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let final_composited =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::FINAL_COMPOSITED);
    assert!(matches!(
        &final_composited.desc,
        RenderGraphResourceDesc::Texture(desc)
            if final_composited.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 320
                && desc.height == 240
                && desc.format == TextureFormat::Rgba8UnormSrgb
    ));
}

#[test]
fn default_core2d_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_core2d();
    let compiled = pipeline.compile(&orthographic_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::Opaque2d,
            RenderPassStage::AlphaMask2d,
            RenderPassStage::Transparent2d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| (pass.name.as_str(), pass.executor_id.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("opaque-sprite", Some("sprite.opaque")),
            ("alpha-mask-sprite", Some("sprite.alpha-mask")),
            ("transparent-sprite", Some("sprite.transparent")),
            (
                "motion-vector-tile-max",
                Some("post.motion-vector-tile-max"),
            ),
            (
                "motion-vector-tile-max-coarse",
                Some("post.motion-vector-tile-max-coarse"),
            ),
            (
                "motion-vector-neighbor-max",
                Some("post.motion-vector-neighbor-max"),
            ),
            (
                "depth-of-field-prepare",
                Some("post.depth-of-field-prepare"),
            ),
            (
                "screen-space-reflection-reflection-pyramid",
                Some("post.screen-space-reflection-reflection-pyramid"),
            ),
            (
                "screen-space-reflection-reflection-pyramid-coarse",
                Some("post.screen-space-reflection-reflection-pyramid-coarse"),
            ),
            (
                "screen-space-reflection-specular-occlusion",
                Some("post.screen-space-reflection-specular-occlusion"),
            ),
            (
                "screen-space-reflection-resolve",
                Some("post.screen-space-reflection-resolve"),
            ),
            ("uber", Some("post.uber")),
            ("runtime-ui", Some("ui.screen-space")),
            ("overlay-gizmo", Some("overlay.gizmo")),
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "post_process".to_string(),
            "sprites".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(compiled.history_bindings, Vec::<FrameHistoryBinding>::new());
}

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
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
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
            "fxaa",
            "overlay-gizmo",
            "runtime-ui",
        ]
    );
}

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
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(live_pass_names.contains(&"taa-resolve"));
    assert!(live_pass_names.contains(&"taa-reactive-mask-clear"));
    assert!(live_pass_names.contains(&"taa-reactive-mask-mesh"));
    assert!(live_pass_names.contains(&"velocity-object"));
    assert!(live_pass_names.contains(&"velocity-camera"));
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
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-resolve")
        .expect("TAA resolve pass should be compiled when TAA is effective");
    assert_eq!(
        taa_pass.executor_id.as_deref(),
        Some("temporal.taa-resolve")
    );
    let reactive_mask_clear_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-reactive-mask-clear")
        .expect("TAA reactive mask clear pass should be compiled when TAA is effective");
    assert_eq!(
        reactive_mask_clear_pass.executor_id.as_deref(),
        Some("temporal.taa-reactive-mask-clear")
    );
    let reactive_mask_clear_write = pass_resource_access(
        &compiled,
        "taa-reactive-mask-clear",
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        reactive_mask_clear_write.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        reactive_mask_clear_write.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    let reactive_mask_mesh_pass = compiled
        .graph
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
        Some(RenderGraphAttachmentOps::load_store())
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
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    let lifetimes = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .map(|lifetime| lifetime.name.as_str())
        .collect::<Vec<_>>();

    assert!(!live_pass_names.contains(&"taa-resolve"));
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
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "preview sky should clear scene depth before depth prepass writes geometry"
    );

    let preview_sky_scene_color = pass_resource_access(
        &compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "preview sky should clear scene color before drawing the background"
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
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "depth prepass should load depth after preview sky initialized the target"
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
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "opaque scene color should load the preview sky background"
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
    let preview_sky_final_color = pass_resource_access(
        &deferred_compiled,
        "preview-sky",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_final_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred preview sky should explicitly clear the imported final-color background target"
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
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred depth prepass should load depth after preview sky initialized the target"
    );
}

#[test]
fn forward_plus_pipeline_compilation_is_deterministic() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let extract = test_extract();

    let first = pipeline.compile(&extract).unwrap();
    let second = pipeline.compile(&extract).unwrap();

    assert_eq!(first, second);
}

#[test]
fn builtin_pipeline_lookup_exposes_deferred_pipeline_handle() {
    let builtin =
        RenderPipelineAsset::builtin(crate::core::framework::render::RenderPipelineHandle::new(2))
            .expect("handle 2 should map to the built-in deferred pipeline");

    assert_eq!(builtin, RenderPipelineAsset::default_deferred());
}

#[test]
fn history_binding_accessors_construct_expected_bindings() {
    assert_eq!(
        FrameHistoryBinding::read(FrameHistorySlot::AmbientOcclusion),
        FrameHistoryBinding {
            slot: FrameHistorySlot::AmbientOcclusion,
            access: FrameHistoryAccess::Read,
        }
    );
    assert_eq!(
        FrameHistoryBinding::write(FrameHistorySlot::TaaSceneColor),
        FrameHistoryBinding {
            slot: FrameHistorySlot::TaaSceneColor,
            access: FrameHistoryAccess::Write,
        }
    );
}

#[test]
fn compile_options_can_disable_clustered_history_and_rendering_plugin_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::Temporal)
        .with_plugin_feature_disabled("screen_space_ambient_occlusion");

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"light-grid-build"));
    assert!(!pass_names.contains(&"taa-resolve"));
    assert!(!compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion
        )));
}

#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-occlusion-cull"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-build"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "light-grid-build"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    let light_zbins_output = pass_resource_access(
        &compiled,
        "light-grid-build",
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        RenderGraphResourceAccessKind::Write,
    );
    let light_tile_masks_output = pass_resource_access(
        &compiled,
        "light-grid-build",
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        light_zbins_output.kind,
        RenderGraphResourceKind::TransientBuffer
    );
    assert_eq!(
        light_tile_masks_output.kind,
        RenderGraphResourceKind::TransientBuffer
    );
    let ssao_output = pass_resource_access(
        &compiled,
        "ssao-evaluate",
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(ssao_output.kind, RenderGraphResourceKind::External);
    assert_eq!(
        ssao_output.attachment_ops, None,
        "compute storage writes must not inherit render attachment load/store ops"
    );
    assert_eq!(compiled.graph.stats().queue_fallback_pass_count, 4);
}

#[test]
fn compile_options_gate_hzb_occlusion_cull_without_removing_hzb_build() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default().with_hzb_occlusion_culling(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert!(!compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-occlusion-cull"));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-build"));
    assert_eq!(compiled.pass_stage("hzb-occlusion-cull"), None);
    assert_eq!(
        compiled.pass_stage("hzb-build"),
        Some(RenderPassStage::AmbientOcclusion)
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceAccessKind::Write,
    );
}

#[test]
fn feature_pass_descriptors_drive_executor_ids_and_resource_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    let depth_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "depth-prepass")
        .expect("default forward pipeline should include depth prepass");
    assert_eq!(
        depth_pass.executor_id.as_deref(),
        Some("mesh.depth-prepass")
    );
    assert!(
        depth_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::GBUFFER_NORMAL
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "depth prepass should declare the normal target it writes"
    );

    let preview_sky_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "preview-sky")
        .expect("default forward pipeline should include preview sky pass");
    assert_eq!(
        preview_sky_pass.executor_id.as_deref(),
        Some("sky.preview-scene-color")
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "preview sky should initialize scene color through the render graph"
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "preview sky should initialize scene depth through the render graph"
    );

    let velocity_object_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "velocity-object")
        .expect("default forward pipeline should include object velocity pass");
    assert_eq!(
        velocity_object_pass.executor_id.as_deref(),
        Some("temporal.velocity-object")
    );
    assert!(
        velocity_object_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "object velocity pass should read scene depth for depth-tested dynamic object writes"
    );
    assert!(
        velocity_object_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                && resource.kind == RenderGraphResourceKind::TransientTexture
                && resource.access == RenderGraphResourceAccessKind::Write
                && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
        }),
        "object velocity pass should initialize the graph-owned velocity target"
    );

    let velocity_camera_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "velocity-camera")
        .expect("default forward pipeline should include camera velocity pass");
    assert_eq!(
        velocity_camera_pass.executor_id.as_deref(),
        Some("temporal.velocity-camera")
    );
    assert!(
        velocity_camera_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "camera velocity pass should read scene depth for per-pixel reconstruction"
    );
    assert!(
        velocity_camera_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                && resource.kind == RenderGraphResourceKind::TransientTexture
                && resource.access == RenderGraphResourceAccessKind::Write
                && resource.attachment_ops == Some(RenderGraphAttachmentOps::load_store())
        }),
        "camera velocity pass should load the object velocity target before filling static pixels"
    );

    let motion_vector_tile_max_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-tile-max")
        .expect("default forward pipeline should include motion-vector tile reconstruction pass");
    assert_eq!(
        motion_vector_tile_max_pass.executor_id.as_deref(),
        Some("post.motion-vector-tile-max")
    );
    assert!(
        motion_vector_tile_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "motion-vector tile reconstruction should read the raw scene motion-vector target"
    );
    assert!(
        motion_vector_tile_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "motion-vector tile reconstruction should write the graph-owned tile-max target"
    );

    let motion_vector_tile_max_coarse_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-tile-max-coarse")
        .expect(
            "default forward pipeline should include coarse motion-vector tile reconstruction pass",
        );
    assert_eq!(
        motion_vector_tile_max_coarse_pass.executor_id.as_deref(),
        Some("post.motion-vector-tile-max-coarse")
    );
    assert!(
        motion_vector_tile_max_coarse_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "coarse motion-vector tile reconstruction should read the first tile-max target"
    );
    assert!(
        motion_vector_tile_max_coarse_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "coarse motion-vector tile reconstruction should write the graph-owned coarse tile-max target"
    );

    let motion_vector_neighbor_max_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-neighbor-max")
        .expect("default forward pipeline should include motion-vector reconstruction pass");
    assert_eq!(
        motion_vector_neighbor_max_pass.executor_id.as_deref(),
        Some("post.motion-vector-neighbor-max")
    );
    assert!(
        motion_vector_neighbor_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "motion-vector reconstruction should read the coarse tile-max motion-vector target"
    );
    assert!(
        motion_vector_neighbor_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "motion-vector reconstruction should write the graph-owned neighbor-max target"
    );

    let opaque_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque-mesh")
        .expect("default forward pipeline should include opaque mesh pass");
    assert_eq!(opaque_pass.executor_id.as_deref(), Some("mesh.opaque"));
    let overlay_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "overlay-gizmo")
        .expect("default forward pipeline should include overlay pass");
    assert!(
        overlay_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "overlay executor should declare its depth read instead of borrowing the target privately"
    );

    let lifetimes = compiled.graph.resource_lifetimes();
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-depth" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-color" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::SCENE_VELOCITY
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors() {
    let mut extract = extract_with_camera(ViewportCameraSnapshot {
        hdr: true,
        msaa_samples: 4,
        ..ViewportCameraSnapshot::default()
    });
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract should carry a selected camera descriptor")
        .target = RenderCameraTarget::Headless {
        size: UVec2::new(1280, 720),
    };
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let scene_color = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-color")
        .expect("scene-color should be a graph resource");
    assert!(matches!(
        &scene_color.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Rg11b10Ufloat
                && desc.sample_count == 4
    ));

    let scene_depth = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-depth")
        .expect("scene-depth should be a graph resource");
    assert!(matches!(
        &scene_depth.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Depth32Float
                && desc.sample_count == 4
    ));

    for (resource_name, expected_width, expected_height, expected_format) in [
        (
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            1280,
            720,
            TextureFormat::Rg16Float,
        ),
        (
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            1280,
            720,
            TextureFormat::Rgba8UnormSrgb,
        ),
        (
            PostProcessGraphResourceNames::HZB_FURTHEST,
            1024,
            512,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            1280,
            720,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
            640,
            360,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
            320,
            180,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            1280,
            720,
            TextureFormat::Rgba8Unorm,
        ),
    ] {
        let lifetime = graph_resource_lifetime(&compiled, resource_name);
        assert!(matches!(
            &lifetime.desc,
            RenderGraphResourceDesc::Texture(desc)
                if desc.width == expected_width
                    && desc.height == expected_height
                    && desc.format == expected_format
                    && desc.sample_count == 1
        ));
    }
}

fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

fn rendering_ssao_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("ao.ssao-evaluate")
        .with_compute_workload(RenderGraphComputeWorkload::viewport(
            "zircon-ssao-pipeline",
            [8, 8, 1],
        ))
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
        .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)],
    )
}

fn rendering_reflection_probes_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_baked_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "baked_lighting",
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.baked-composite")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_post_process_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-tile-max",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-tile-max")
            .read_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-tile-max-coarse",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-tile-max-coarse")
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-neighbor-max",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-neighbor-max")
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "depth-of-field-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("post.depth-of-field-prepare")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_texture_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-reflection-pyramid",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-reflection-pyramid")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-reflection-pyramid-coarse",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-reflection-pyramid-coarse")
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-specular-occlusion",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-specular-occlusion")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-resolve")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
            .read_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
            )
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION)
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "uber",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .read_texture(PostProcessGraphResourceNames::BLOOM)
            .read_texture(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)
            .read_texture(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
            .write_texture(PostProcessGraphResourceNames::FINAL_COMPOSITED)
            .write_external(PostProcessGraphResourceNames::FINAL_COLOR)
            .write_texture(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
        ],
    )
}

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
        .graph
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
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(live_pass_names.contains(&"uber"));
    for pass_name in [
        "velocity-object",
        "velocity-camera",
        "taa-reactive-mask-clear",
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
        .graph
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
        .graph
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
        .graph
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
        .graph
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
        .enabled_features
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
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"bloom-extract"));
    assert!(pass_names.contains(&"custom-bloom-pass"));
    assert!(compiled
        .required_extract_sections
        .contains(&"custom_post".to_string()));
    assert!(compiled.graph.resource_lifetimes().iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque,
                "custom-gbuffer-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .with_side_effects()],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "uber",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .with_side_effects()],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .write_buffer("scene-color")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-external-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .write_external("scene-color")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .write_texture("")],
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
            attachment_ops: None,
            write_mode: RenderFeatureResourceWriteMode::Storage,
            external_binding: crate::render_graph::RenderGraphExternalResourceBinding::report_only(
            ),
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "post-stack-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .read_texture("scene-color")],
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
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "history-using-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .read_texture("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate history binding for slot `TaaSceneColor`"),
        "unexpected error: {error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    extract_with_camera(ViewportCameraSnapshot::default())
}

fn extract_with_camera(camera: ViewportCameraSnapshot) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn pass_resource_access<'a>(
    compiled: &'a crate::graphics::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a crate::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .and_then(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource.name == resource_name && resource.access == access)
        })
        .unwrap_or_else(|| panic!("pass `{pass_name}` should {access:?} `{resource_name}`"))
}

fn graph_resource_lifetime<'a>(
    compiled: &'a crate::graphics::CompiledRenderPipeline,
    resource_name: &str,
) -> &'a crate::render_graph::RenderGraphResourceLifetime {
    compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("compiled graph should contain resource `{resource_name}`"))
}

fn orthographic_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Orthographic,
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
