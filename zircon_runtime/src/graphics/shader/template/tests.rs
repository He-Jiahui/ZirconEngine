use crate::core::framework::render::{
    builtin_geometry_source_descriptor, GBufferChannelMask, GeometrySourceDescriptor,
    RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialFallbackPolicy,
    RenderMaterialLightingModel, RenderMaterialTextureTransform, RenderQueueValue,
    ShaderFeatureBits, ShaderPassType, ShadingModelDescriptor, ShadingModelId,
    StandardMaterialDescriptor, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::{AssetReference, ResourceLocator};

use super::assemble::{
    assemble_material_shader_template, MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
};
use super::deferred_gbuffer::{
    assemble_deferred_gbuffer_shader_template, DeferredGBufferShaderTemplateRequest,
};
use super::material_surface::{
    standard_material_surface_source, standard_material_surface_source_for_features,
};
use super::pass_specialization::pass_template_for;
use super::validation::validate_material_shader_template_wgsl;

const MATERIAL_SURFACE: &str = r#"
fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(vec4<f32>(input.color.rgb, 1.0));
}
"#;

const CUSTOM_TOON_FORWARD_INCLUDE: &str = r#"
const ZR_SHADING_TOON_DEBUG_ID: u32 = 16u;

fn zr_toon_band(normal_ws: vec3<f32>) -> f32 {
    return select(0.35, 0.85, normal_ws.z >= 0.0);
}

fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    let toon_band = zr_toon_band(surface.normal_ws);
    return surface.base_color.rgb * toon_band + surface.emissive + vec3<f32>(ctx.frag_coord.x * 0.0);
}
"#;

const CUSTOM_TOON_GBUFFER_INCLUDE: &str = r#"
const ZR_GBUFFER_TOON_DEBUG_ID: u32 = 16u;

fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), surface.base_color.a),
        vec4<f32>(
            0.25,
            0.75,
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(ZR_GBUFFER_TOON_DEBUG_ID, receive_shadows),
        ),
        vec4<f32>(surface.emissive, 1.0),
    );
}
"#;

fn static_mesh_descriptor() -> GeometrySourceDescriptor {
    builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH).expect("static descriptor")
}

fn material_template_request(
    geometry_source: GeometrySourceDescriptor,
    pass_type: ShaderPassType,
) -> MaterialShaderTemplateRequest {
    MaterialShaderTemplateRequest::new(geometry_source, pass_type, MATERIAL_SURFACE, "user_surface")
}

fn has_include_token(tokens: &[String], token: &str) -> bool {
    tokens.iter().any(|include_token| include_token == token)
}

macro_rules! assert_include_token {
    ($assembly:expr, $token:expr) => {
        assert!(has_include_token(&($assembly).include_tokens, $token));
    };
}

macro_rules! assert_missing_include_token {
    ($assembly:expr, $token:expr) => {
        assert!(!has_include_token(&($assembly).include_tokens, $token));
    };
}

#[path = "tests/standard_material_surface_template.rs"]
mod standard_material_surface_template;
#[path = "tests/surface_modules.rs"]
mod surface_modules;

fn toon_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        ShadingModelId::new(16),
        "toon",
        "zr_shading_toon.wgsl",
        "zr_gbuffer_encode_toon.wgsl",
        "zr_shade_deferred_toon.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

#[test]
fn render_shader_template_assembles_static_and_skinned_geometry_sources() {
    let static_mesh = static_mesh_descriptor();
    let skinned_mesh = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        .expect("skinned descriptor");

    let static_assembly = assemble_material_shader_template(material_template_request(
        static_mesh,
        ShaderPassType::Forward,
    ))
    .expect("static template assembly");
    let skinned_assembly = assemble_material_shader_template(material_template_request(
        skinned_mesh,
        ShaderPassType::Forward,
    ))
    .expect("skinned template assembly");

    assert_ne!(static_assembly.wgsl_source, skinned_assembly.wgsl_source);
    assert_include_token!(static_assembly, "zr_geometry_static.wgsl");
    assert_include_token!(static_assembly, "zr_scene_runtime.wgsl");
    assert_include_token!(static_assembly, "zr_gpu_scene.wgsl");
    assert_include_token!(static_assembly, "zr_environment.wgsl");
    assert_include_token!(skinned_assembly, "zr_geometry_skinned.wgsl");
    assert!(static_assembly
        .wgsl_source
        .contains("fn zr_material_surface("));
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main("));
    assert!(static_assembly.wgsl_source.contains("fn vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn fs_main("));
    assert!(static_assembly
        .wgsl_source
        .contains("return zr_vs_main_impl(v, instance_index);"));
    assert!(static_assembly
        .wgsl_source
        .contains("return zr_fs_main_impl(input);"));
    assert!(static_assembly
        .wgsl_source
        .contains("ZR_GEOMETRY_SOURCE_STATIC_MESH"));
    assert!(static_assembly
        .wgsl_source
        .contains("@group(0) @binding(0) var<uniform> scene: SceneUniform"));
    assert!(static_assembly
        .wgsl_source
        .contains("@group(3) @binding(1) var<storage, read> zr_instance_data"));
    assert!(static_assembly
        .wgsl_source
        .contains("let world_from_local = zr_world_from_local(instance_index);"));
    assert!(static_assembly
        .wgsl_source
        .contains("output.clip_position = scene.view_proj * position_ws;"));
    assert!(skinned_assembly
        .wgsl_source
        .contains("zr_skinned_joint_matrix(v.joints.x)"));
    assert!(skinned_assembly
        .wgsl_source
        .contains("@group(3) @binding(3) var<uniform> zr_skinned_joint_palette"));
    assert!(!skinned_assembly
        .wgsl_source
        .contains("@group(3) @binding(1) var<storage, read> zr_joint_palette"));
    validate_material_shader_template_wgsl(&static_assembly.wgsl_source)
        .expect("static template WGSL should validate");
    validate_material_shader_template_wgsl(&skinned_assembly.wgsl_source)
        .expect("skinned template WGSL should validate");
    assert_eq!(
        static_assembly.include_content_hashes.len(),
        static_assembly.include_tokens.len()
    );
    assert_eq!(static_assembly.template_revision, "zr-material-template-v1");
}

#[test]
fn render_shader_template_validates_morphed_geometry_sources_with_payload_slots() {
    for (geometry_source_id, include_token, source_define) in [
        (
            GEOMETRY_SOURCE_ID_MORPHED_MESH,
            "zr_geometry_morphed.wgsl",
            "ZR_GEOMETRY_SOURCE_MORPHED_MESH",
        ),
        (
            GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
            "zr_geometry_skinned_morphed.wgsl",
            "ZR_GEOMETRY_SOURCE_SKINNED_MORPHED_MESH",
        ),
    ] {
        let geometry_source = builtin_geometry_source_descriptor(geometry_source_id)
            .expect("morphed geometry source descriptor");

        for pass_type in [
            ShaderPassType::Forward,
            ShaderPassType::GBuffer,
            ShaderPassType::DepthPrepass,
            ShaderPassType::Shadow,
            ShaderPassType::Velocity,
            ShaderPassType::TaaReactiveMask,
        ] {
            let assembly = match pass_type {
                ShaderPassType::TaaReactiveMask => {
                    let material = standard_material_descriptor();
                    let surface_source = standard_material_surface_source(&material);
                    assemble_material_shader_template(
                        MaterialShaderTemplateRequest::new(
                            geometry_source.clone(),
                            pass_type,
                            surface_source.source.clone(),
                            surface_source.entry_point,
                        )
                        .with_features(surface_source.features),
                    )
                }
                ShaderPassType::Forward
                | ShaderPassType::GBuffer
                | ShaderPassType::DepthPrepass
                | ShaderPassType::Shadow
                | ShaderPassType::Velocity => assemble_material_shader_template(
                    material_template_request(geometry_source.clone(), pass_type),
                ),
            }
            .expect("morphed template assembly");

            assert_include_token!(assembly, include_token);
            assert!(assembly.wgsl_source.contains(source_define));
            assert!(assembly
                .wgsl_source
                .contains("@builtin(vertex_index) vertex_index: u32"));
            assert!(assembly.wgsl_source.contains("morph_payload_slot"));
            assert!(assembly.wgsl_source.contains("zr_gpu_scene_morph_payload"));
            assert!(assembly.wgsl_source.contains("zr_morph_previous_weight"));
            assert!(assembly
                .wgsl_source
                .contains("payload.y + payload.w + target_index"));
            validate_material_shader_template_wgsl(&assembly.wgsl_source)
                .expect("morphed template WGSL should validate");
        }
    }
}

#[test]
fn standard_material_surface_source_can_be_built_from_runtime_features() {
    let features = ShaderFeatureBits::new(
        ShaderFeatureBits::ALPHA_TEST
            | ShaderFeatureBits::RECEIVE_SHADOWS
            | ShaderFeatureBits::DOUBLE_SIDED,
    );
    let surface_source = standard_material_surface_source_for_features(features, 1.25);

    assert_eq!(surface_source.entry_point, "standard_material_surface");
    assert_eq!(surface_source.features, features);
    assert!(surface_source
        .source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 1.00000000;"));
    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS));

    let nan_surface_source =
        standard_material_surface_source_for_features(ShaderFeatureBits::default(), f32::NAN);
    assert!(nan_surface_source
        .source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.00000000;"));
}

#[test]
fn render_shader_template_validates_standard_material_wgsl_with_naga() {
    let static_mesh = static_mesh_descriptor();
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Forward,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard material template assembly");

    let validation = validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("assembled standard material WGSL should validate");

    for expected in ["zr_vs_main", "vs_main", "zr_fs_main", "fs_main"] {
        assert!(
            validation
                .entry_points
                .iter()
                .any(|entry_point| entry_point == expected),
            "assembled standard material WGSL should expose `{expected}`"
        );
    }
}

#[test]
fn render_shader_template_clips_alpha_for_masked_standard_material_passes() {
    let static_mesh = static_mesh_descriptor();
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);

    let depth_alpha = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh.clone(),
            ShaderPassType::DepthPrepass,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("alpha depth template assembly");
    let shadow_alpha = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Shadow,
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("alpha shadow template assembly");

    for source in [&depth_alpha.wgsl_source, &shadow_alpha.wgsl_source] {
        assert!(source.contains("@fragment"));
        assert!(source.contains("let surface = zr_material_surface(input);"));
        assert!(source.contains("zr_apply_alpha_clip(surface);"));
        assert!(source.contains("fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)"));
        assert!(source.contains("surface.alpha_cutoff = standard_material_alpha_cutoff();"));
        assert!(source.contains("standard_material_properties.data8.z"));
        assert!(source.contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
        assert!(source.contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
    }
    assert_include_token!(depth_alpha, "zr_template_depth_alpha.wgsl");
    assert_include_token!(shadow_alpha, "zr_template_shadow_alpha.wgsl");
}

#[test]
fn render_shader_template_specializes_depth_and_velocity_passes() {
    let static_mesh = static_mesh_descriptor();

    let depth_no_alpha = assemble_material_shader_template(material_template_request(
        static_mesh.clone(),
        ShaderPassType::DepthPrepass,
    ))
    .expect("depth template assembly");
    let depth_alpha = assemble_material_shader_template(
        material_template_request(static_mesh.clone(), ShaderPassType::DepthPrepass)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST)),
    )
    .expect("alpha depth template assembly");
    let velocity = assemble_material_shader_template(material_template_request(
        static_mesh.clone(),
        ShaderPassType::Velocity,
    ))
    .expect("velocity template assembly");
    let velocity_alpha = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Velocity)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST)),
    )
    .expect("alpha velocity template assembly");

    assert_include_token!(depth_no_alpha, "zr_template_depth.wgsl");
    assert!(!depth_no_alpha.wgsl_source.contains("zr_material_surface"));
    assert!(!depth_no_alpha.wgsl_source.contains("@fragment"));
    assert!(!depth_no_alpha
        .wgsl_source
        .contains("surface.normal_ws * 0.5"));
    assert!(!depth_no_alpha
        .wgsl_source
        .contains("zr_template_gbuffer.wgsl"));
    assert!(depth_alpha.wgsl_source.contains("zr_material_surface"));
    assert_include_token!(depth_alpha, "zr_template_depth_alpha.wgsl");
    assert!(depth_alpha
        .wgsl_source
        .contains("zr_apply_alpha_clip(surface);"));
    assert!(!depth_alpha.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(!depth_alpha.wgsl_source.contains("zr_template_gbuffer.wgsl"));
    assert!(velocity.wgsl_source.contains("fetch_prev_position"));
    assert!(velocity
        .wgsl_source
        .contains("struct ZrVelocityVertexInput"));
    assert!(velocity
        .wgsl_source
        .contains("@location(8) previous_position"));
    assert!(velocity
        .wgsl_source
        .contains("let previous_input = zr_velocity_vertex_input(v, v.previous_position);"));
    assert!(velocity.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn vs_main("));
    assert!(velocity.wgsl_source.contains("fn fs_main("));
    assert!(velocity
        .wgsl_source
        .contains("scene.view_proj_unjittered * current_world"));
    assert!(velocity
        .wgsl_source
        .contains("scene.previous_view_proj_unjittered * previous_world"));
    assert!(velocity
        .wgsl_source
        .contains("return zr_vs_main_impl(v, instance_index);"));
    assert!(!velocity.wgsl_source.contains("zr_material_surface"));
    assert!(velocity_alpha.wgsl_source.contains("zr_material_surface"));
    assert!(velocity_alpha
        .wgsl_source
        .contains("zr_surface_fails_alpha_clip(surface)"));
    assert_include_token!(velocity_alpha, "zr_template_velocity_alpha.wgsl");
    validate_material_shader_template_wgsl(&depth_no_alpha.wgsl_source)
        .expect("assembled depth-only WGSL should validate");
    validate_material_shader_template_wgsl(&depth_alpha.wgsl_source)
        .expect("assembled alpha depth-only WGSL should validate");
    validate_material_shader_template_wgsl(&velocity.wgsl_source)
        .expect("assembled velocity WGSL should validate");
    validate_material_shader_template_wgsl(&velocity_alpha.wgsl_source)
        .expect("assembled alpha velocity WGSL should validate");

    let velocity_pass = pass_template_for(ShaderPassType::Velocity, ShaderFeatureBits::default());
    let velocity_alpha_pass = pass_template_for(
        ShaderPassType::Velocity,
        ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
    );
    assert!(velocity_pass.uses_previous_position);
    assert!(!velocity_pass.requires_material_surface);
    assert!(velocity_alpha_pass.uses_previous_position);
    assert!(velocity_alpha_pass.requires_material_surface);
}

#[test]
fn render_shader_template_uses_shading_model_descriptor_forward_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = ShadingModelDescriptor::new(
        SHADING_MODEL_ID_STANDARD_PBR,
        "standard_pbr",
        "zr_shading_standard_pbr.wgsl",
        "zr_gbuffer_encode_standard_pbr.wgsl",
        "zr_shade_deferred_standard_pbr.wgsl",
        GBufferChannelMask::standard_lit(),
    );

    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor),
    )
    .expect("descriptor-backed forward template assembly");

    assert_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
    assert_include_token!(assembly, "zr_environment.wgsl");
    assert_eq!(
        assembly
            .include_tokens
            .iter()
            .filter(|token| token.as_str() == "zr_shading_standard_pbr.wgsl")
            .count(),
        1
    );
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_shading_standard_pbr.wgsl"));
    assert!(assembly
        .wgsl_source
        .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)"));
}

#[test]
fn render_shader_template_rejects_unknown_shading_model_forward_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let error = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor),
    )
    .expect_err("unknown descriptor include should fail before template assembly succeeds");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: "zr_shading_toon.wgsl".to_string(),
        }
    );
}

#[test]
fn render_shader_template_uses_custom_shading_model_forward_include_source() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_forward_include_source(
                "zr_shading_toon.wgsl",
                CUSTOM_TOON_FORWARD_INCLUDE,
            ),
    )
    .expect("custom descriptor-backed forward template assembly");

    assert_include_token!(assembly, "zr_shading_toon.wgsl");
    assert_missing_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_shading_toon.wgsl"));
    assert!(assembly.wgsl_source.contains("ZR_SHADING_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn zr_toon_band"));
    assert!(assembly
        .wgsl_source
        .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)"));
    assert_include_token!(assembly, "zr_environment.wgsl");
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("custom shading include template WGSL should validate");
}

#[test]
fn render_deferred_gbuffer_template_rejects_unknown_shading_model_gbuffer_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let error = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(static_mesh, MATERIAL_SURFACE, "user_surface")
            .with_shading_model_descriptor(descriptor),
    )
    .expect_err("unknown descriptor GBuffer include should fail before template assembly succeeds");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: "zr_gbuffer_encode_toon.wgsl".to_string(),
        }
    );
}

#[test]
fn render_deferred_gbuffer_template_uses_custom_shading_model_gbuffer_include_source() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let assembly = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(static_mesh, MATERIAL_SURFACE, "user_surface")
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_gbuffer_include_source(
                "zr_gbuffer_encode_toon.wgsl",
                CUSTOM_TOON_GBUFFER_INCLUDE,
            ),
    )
    .expect("custom descriptor-backed deferred GBuffer template assembly");

    assert_include_token!(assembly, "zr_gbuffer_encode_toon.wgsl");
    assert_missing_include_token!(assembly, "zr_gbuffer_encode_standard_pbr.wgsl");
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_gbuffer_encode_toon.wgsl"));
    assert!(assembly.wgsl_source.contains("ZR_GBUFFER_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn encode_gbuffer"));
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_template_deferred_gbuffer.wgsl"));
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("custom deferred GBuffer include template WGSL should validate");
}

fn standard_material_descriptor() -> StandardMaterialDescriptor {
    let shader_reference = AssetReference::from_locator(
        ResourceLocator::parse("builtin://shader/pbr.wgsl").expect("shader locator"),
    );
    StandardMaterialDescriptor {
        name: Some("template-standard-material".to_string()),
        dependencies: RenderMaterialDependencySet::new(shader_reference),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        base_color_texture_transform: RenderMaterialTextureTransform::default(),
        base_color_texture_uv_channel: 0,
        normal_texture: None,
        normal_texture_transform: RenderMaterialTextureTransform::default(),
        normal_texture_uv_channel: 0,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        metallic_roughness_texture_transform: RenderMaterialTextureTransform::default(),
        metallic_roughness_texture_uv_channel: 0,
        occlusion_texture: None,
        occlusion_texture_transform: RenderMaterialTextureTransform::default(),
        occlusion_texture_uv_channel: 0,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        emissive_texture_transform: RenderMaterialTextureTransform::default(),
        emissive_texture_uv_channel: 0,
        alpha_mode: RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
        lighting_model: RenderMaterialLightingModel::Pbr,
        unlit: false,
        double_sided: true,
        cast_shadows: true,
        receive_shadows: true,
        render_queue: 2450,
        render_queue_value: Some(RenderQueueValue::ALPHA_TEST),
        material_queue: 0,
        depth_bias: 0.0,
        taa_reactive_mask_strength: 0.0,
        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
    }
}

#[test]
fn render_shader_template_rejects_reserved_material_symbols() {
    let static_mesh = static_mesh_descriptor();
    let error = assemble_material_shader_template(MaterialShaderTemplateRequest::new(
        static_mesh,
        ShaderPassType::Forward,
        "fn fetch_position(input: ZrVertexOutput) -> ZrSurfaceOutput { return zr_surface_from_base_color(input.color); }",
        "fetch_position",
    ))
    .expect_err("reserved symbol should fail");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::ReservedMaterialSymbol {
            symbol: "fetch_position".to_string(),
            prefix: "fetch_",
        }
    );
}
