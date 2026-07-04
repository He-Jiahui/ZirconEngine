use super::*;
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, ShaderFeatureBits, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::graphics::shader::{
    assemble_deferred_gbuffer_shader_template, standard_material_surface_source_for_features,
    DeferredGBufferShaderTemplateRequest,
};

fn deferred_gbuffer_test_shader() -> String {
    let features = ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS);
    let material_surface = standard_material_surface_source_for_features(features, 0.0);
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static mesh geometry source should be registered");

    assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            geometry_source,
            material_surface.source,
            material_surface.entry_point,
        )
        .with_features(material_surface.features),
    )
    .expect("deferred G-buffer shader template should assemble for static meshes")
    .wgsl_source
}

#[test]
fn deferred_material_gbuffer_shaders_encode_and_decode_material_channels() {
    let geometry_shader = deferred_gbuffer_test_shader();
    let lighting_shader = concat!(
        include_str!("../../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl"),
        "\n",
        include_str!("../../scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"),
        "\n",
        include_str!("../../scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
        "\n",
        include_str!("../../shader/wgsl/zr_environment.wgsl"),
        "\n",
        include_str!("../../shader/wgsl/zr_shade_deferred_standard_pbr.wgsl"),
        "\n",
        include_str!("../../shader/wgsl/zr_shade_deferred_blinn_phong.wgsl"),
        "\n",
        include_str!("../../shader/wgsl/zr_shade_deferred_unlit.wgsl"),
        "\n",
        include_str!("../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl")
    );
    for (name, shader) in [
        (
            "zr_template_deferred_gbuffer.wgsl",
            geometry_shader.as_str(),
        ),
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
        geometry_shader.contains("@location(1) normal: vec4<f32>")
            && geometry_shader.contains("@location(2) material: vec4<f32>"),
        "deferred geometry should emit normal and material G-buffer targets"
    );
    assert!(
        geometry_shader.contains("standard_material_properties.data0.x")
            && geometry_shader.contains("standard_material_properties.data0.y")
            && geometry_shader.contains("surface.metallic")
            && geometry_shader.contains("surface.roughness"),
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
    assert!(
        geometry_shader.contains(
            "zr_deferred_encode_material_flags(surface.shading_model_id, receive_shadows)"
        )
            && lighting_shader.contains("let receive_shadows = decode_receive_shadows(material.a);")
            && lighting_shader.contains("if (receive_shadows)"),
        "deferred material G-buffer alpha should preserve receive-shadow state for deferred lighting"
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
            ("output-transfer", Some("post.output-transfer")),
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
