use crate::core::framework::render::{
    GBufferChannelMask, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GeometrySourceDescriptor, RenderMaterialAlphaMode, RenderMaterialDependencySet,
    RenderMaterialFallbackPolicy, RenderMaterialLightingModel, RenderMaterialTextureTransform,
    RenderQueueValue, SHADING_MODEL_ID_STANDARD_PBR, ShaderFeatureBits, ShaderPassType,
    ShadingModelDescriptor, ShadingModelId, StandardMaterialDescriptor,
    StandardPbrMaterialFeatures, builtin_geometry_source_descriptor,
};
use crate::core::resource::{AssetReference, ResourceLocator};

use super::assemble::{
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError, assemble_material_shader_template,
};
use super::deferred_gbuffer::{
    DeferredGBufferShaderTemplateRequest, assemble_deferred_gbuffer_shader_template,
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

const STANDARD_PBR_FORWARD_SHADER: &str = include_str!("../wgsl/zr_shading_standard_pbr.wgsl");

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

#[test]
fn forward_material_template_applies_integrated_volumetric_lighting() {
    let static_mesh = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static mesh geometry source");
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh,
        ShaderPassType::Forward,
    ))
    .expect("forward material shader template should assemble");

    assert_include_token!(assembly, "zr_volumetric.wgsl");
    assert!(assembly
        .wgsl_source
        .contains("zr_volumetric_apply(shaded + baked_indirect, input.clip_position.xy, input.clip_position.z)"));
}

#[test]
fn standard_pbr_direct_lighting_normalizes_surface_inputs_once_per_pixel() {
    let light_loop = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_gpu_light_lighting(")
        .nth(1)
        .and_then(|source| source.split("fn shade_forward(").next())
        .expect("standard PBR should retain the GPU light-grid owner");
    for expected in [
        "let world_normal = zr_normalize_or_zero(surface.normal_ws);",
        "let world_view = zr_normalize_or_zero(view_dir_ws);",
        "if (surface.shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "let direct_metallic = clamp(surface.metallic, 0.0, 1.0);",
        "direct_f0 = mix(",
        "direct_metallic,",
        "direct_diffuse_brdf =",
        "direct_base_energy = zr_pbr_clearcoat_base_energy_scale(surface, world_view);",
        "direct_clearcoat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);",
        "world_normal,",
        "world_view,",
        "direct_f0,",
        "direct_diffuse_brdf,",
        "direct_base_energy,",
        "direct_clearcoat_normal,",
    ] {
        assert!(
            light_loop.contains(expected),
            "light-grid setup must retain `{expected}`"
        );
    }
    let direct_material_setup = light_loop
        .split("let direct_metallic = clamp(surface.metallic, 0.0, 1.0);")
        .nth(1)
        .and_then(|source| source.split("if (ZR_FEATURE_PBR_CLEARCOAT").next())
        .expect("light-grid setup must derive direct PBR material inputs once");
    for expected in ["direct_metallic,", "(1.0 - direct_metallic)"] {
        assert!(
            direct_material_setup.contains(expected),
            "direct PBR setup must consume the clamped metallic value in `{expected}`"
        );
    }
    assert!(
        !direct_material_setup.contains("surface.metallic"),
        "direct PBR setup must not reuse unbounded surface metallic after normalization"
    );
    let clearcoat_setup_guard = light_loop
        .find("if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat != 0.0) {")
        .expect("light-grid setup must skip clearcoat preparation when it cannot contribute");
    let clearcoat_base_energy = light_loop
        .find("direct_base_energy = zr_pbr_clearcoat_base_energy_scale(surface, world_view);")
        .expect("light-grid setup must prepare clearcoat base energy when required");
    let clearcoat_normal = light_loop
        .find("direct_clearcoat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);")
        .expect("light-grid setup must prepare a clearcoat normal when required");
    assert!(
        clearcoat_setup_guard < clearcoat_base_energy && clearcoat_base_energy < clearcoat_normal,
        "clearcoat preparation must remain inside the feature and material-weight guard"
    );

    let per_light = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_shade_gpu_light_index(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_gpu_light_lighting(")
                .next()
        })
        .expect("standard PBR should retain the per-light owner");
    assert!(
        per_light.contains("zr_standard_pbr_shade_light_vector_normalized("),
        "per-light shading must use normalized inputs supplied by the light-grid owner"
    );
    assert!(
        !per_light.contains("zr_normalize_or_zero(surface.normal_ws)"),
        "per-light shading must not renormalize the surface normal"
    );
    assert!(
        !per_light.contains("zr_normalize_or_zero(view_dir_ws)"),
        "per-light shading must not renormalize the view direction"
    );
    for expected in ["direct_f0", "direct_diffuse_brdf", "direct_base_energy"] {
        assert!(
            per_light.contains(expected),
            "per-light shading must consume the precomputed `{expected}`"
        );
    }
    assert!(
        !per_light.contains("surface.occlusion"),
        "direct-light radiance must not apply ambient occlusion"
    );

    let standard_lobe = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_shade_standard_light_vector_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_shade_blinn_phong_light_vector_normalized(")
                .next()
        })
        .expect("standard PBR must retain the direct-light lobe owner");
    assert!(
        !standard_lobe.contains("let f0 = mix("),
        "the standard PBR lobe must not recompute F0 per light"
    );
    assert!(
        !standard_lobe.contains("zr_pbr_clearcoat_base_energy_scale("),
        "the standard PBR lobe must not recompute clearcoat base energy per light"
    );
    assert!(
        !standard_lobe.contains("zr_normalize_or_zero(surface.clearcoat_normal_ws)"),
        "the standard PBR lobe must not renormalize the clearcoat normal per light"
    );
    assert!(
        !standard_lobe.contains("let specular = select("),
        "the standard PBR lobe must not evaluate isotropic and anisotropic GGX before selecting one"
    );
    for expected in [
        "var specular = zr_pbr_isotropic_ggx(",
        "if (ZR_FEATURE_PBR_ANISOTROPY) {",
        "specular = zr_aniso_ggx(",
    ] {
        assert!(
            standard_lobe.contains(expected),
            "the standard PBR lobe must retain feature-selected anisotropy `{expected}`"
        );
    }
    for (feature_guard, material_weight, lobe) in [
        (
            "if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat != 0.0)",
            "surface.clearcoat",
            "zr_clearcoat_lobe(",
        ),
        (
            "if (ZR_FEATURE_PBR_TRANSMISSION && surface.diffuse_transmission != 0.0)",
            "surface.diffuse_transmission",
            "zr_transmission_btdf(",
        ),
    ] {
        let guard = standard_lobe.find(feature_guard).unwrap_or_else(|| {
            panic!("standard PBR lobe must guard `{lobe}` with `{feature_guard}`")
        });
        let call = standard_lobe
            .find(lobe)
            .unwrap_or_else(|| panic!("standard PBR lobe must retain `{lobe}`"));
        assert!(
            guard < call,
            "standard PBR lobe must skip `{lobe}` when `{material_weight}` cannot contribute"
        );
    }

    let forward_shading = STANDARD_PBR_FORWARD_SHADER
        .split("fn shade_forward(")
        .nth(1)
        .expect("standard PBR must retain forward shading");
    for (feature_guard, helper) in [
        (
            "if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat != 0.0) {",
            "zr_pbr_clearcoat_base_energy_scale(surface, view_dir_ws);",
        ),
        (
            "if (ZR_FEATURE_PBR_TRANSMISSION && surface.specular_transmission > 0.0) {",
            "zr_pbr_screen_space_transmission(",
        ),
    ] {
        let guard = forward_shading
            .find(feature_guard)
            .unwrap_or_else(|| panic!("forward shading must retain `{feature_guard}`"));
        let helper_call = forward_shading
            .find(helper)
            .unwrap_or_else(|| panic!("forward shading must retain `{helper}`"));
        assert!(
            guard < helper_call,
            "forward shading must skip `{helper}` when its feature cannot contribute"
        );
    }
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
    assert_missing_include_token!(static_assembly, "zr_oit.wgsl");
    assert_include_token!(skinned_assembly, "zr_geometry_skinned.wgsl");
    assert!(
        static_assembly
            .wgsl_source
            .contains("fn zr_material_surface(")
    );
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main("));
    assert!(static_assembly.wgsl_source.contains("fn vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn fs_main("));
    assert!(!static_assembly.wgsl_source.contains("fn fs_oit("));
    assert!(!static_assembly.wgsl_source.contains("oit_draw("));
    assert!(
        static_assembly
            .wgsl_source
            .contains("return zr_vs_main_impl(v, instance_index);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("return zr_fs_main_impl(input);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("ZR_GEOMETRY_SOURCE_STATIC_MESH")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("@group(0) @binding(0) var<uniform> scene: SceneUniform")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("@group(3) @binding(1) var<storage, read> zr_instance_data")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("let world_from_local = zr_world_from_local(instance_index);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("output.clip_position = scene.view_proj * position_ws;")
    );
    assert!(
        skinned_assembly
            .wgsl_source
            .contains("zr_skinned_joint_matrix(v.joints.x)")
    );
    assert!(
        skinned_assembly
            .wgsl_source
            .contains("@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette")
    );
    assert!(
        !skinned_assembly
            .wgsl_source
            .contains("@group(3) @binding(1) var<storage, read> zr_joint_palette")
    );
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
fn render_volumetric_forward_shader_variant_removes_bindings_when_disabled() {
    let disabled = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("disabled volumetric forward variant");
    let enabled = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::VOLUMETRIC_FOG)),
    )
    .expect("enabled volumetric forward variant");

    for binding in ["@binding(25)", "@binding(26)", "@binding(27)"] {
        assert!(!disabled.wgsl_source.contains(binding));
        assert!(enabled.wgsl_source.contains(binding));
    }
    assert!(
        disabled
            .wgsl_source
            .contains("const ZR_FEATURE_VOLUMETRIC_FOG: bool = false;")
    );
    assert!(
        enabled
            .wgsl_source
            .contains("const ZR_FEATURE_VOLUMETRIC_FOG: bool = true;")
    );
    validate_material_shader_template_wgsl(&disabled.wgsl_source)
        .expect("disabled volumetric forward WGSL should validate");
    validate_material_shader_template_wgsl(&enabled.wgsl_source)
        .expect("enabled volumetric forward WGSL should validate");
}

#[test]
fn render_transmission_uses_viewport_local_uv_for_nonzero_viewport_origins() {
    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::PBR_TRANSMISSION)),
    )
    .expect("transmission forward variant");

    for expected in [
        "fn zr_pbr_viewport_uv(world_position: vec3<f32>)",
        "scene.view_proj * vec4<f32>(world_position, 1.0)",
        "zr_pbr_viewport_uv(world_position)",
        "ctx.position_ws",
    ] {
        assert!(
            assembly.wgsl_source.contains(expected),
            "transmission source should contain viewport-local contract `{expected}`"
        );
    }
    assert!(
        !assembly
            .wgsl_source
            .contains("fragment_position / max(transmission_extent")
    );
    assert!(
        !assembly
            .wgsl_source
            .contains("textureDimensions(zr_transmission_scene_color)")
    );
    assert!(!assembly.wgsl_source.contains(
        "zr_pbr_screen_space_transmission(\n        surface,\n        ctx.frag_coord.xy"
    ));
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("viewport-local transmission WGSL should validate");
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
            assert!(
                assembly
                    .wgsl_source
                    .contains("@builtin(vertex_index) vertex_index: u32")
            );
            assert!(assembly.wgsl_source.contains("morph_payload_slot"));
            assert!(assembly.wgsl_source.contains("zr_gpu_scene_morph_payload"));
            assert!(assembly.wgsl_source.contains("zr_morph_previous_weight"));
            assert!(
                assembly
                    .wgsl_source
                    .contains("payload.y + payload.w + target_index")
            );
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
    assert!(
        surface_source
            .source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 1.00000000;")
    );
    assert!(
        surface_source
            .features
            .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
    );

    let nan_surface_source =
        standard_material_surface_source_for_features(ShaderFeatureBits::default(), f32::NAN);
    assert!(
        nan_surface_source
            .source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.00000000;")
    );
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

    for expected in [
        "// include: zr_lightmap.wgsl",
        "@group(1) @binding(23) var<storage, read> zr_light_probe_grid",
        "@group(1) @binding(24) var zr_lightmap_atlas: texture_2d_array<f32>;",
        "@group(1) @binding(28) var zr_lightmap_sampler: sampler;",
        "zr_lightmap_baked_irradiance(",
    ] {
        assert!(assembly.wgsl_source.contains(expected));
    }

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
fn standard_pbr_clearcoat_base_energy_variants_validate_with_naga() {
    let clearcoat_disabled = standard_material_descriptor();
    let mut clearcoat_enabled = standard_material_descriptor();
    clearcoat_enabled.advanced_features.clearcoat = 0.75;
    let mut blinn_phong = standard_material_descriptor();
    blinn_phong.lighting_model = RenderMaterialLightingModel::BlinnPhong;

    for (label, material) in [
        ("standard PBR clearcoat disabled", clearcoat_disabled),
        ("standard PBR clearcoat enabled", clearcoat_enabled),
        ("Blinn-Phong", blinn_phong),
    ] {
        let surface_source = standard_material_surface_source(&material);
        let assembly = assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                static_mesh_descriptor(),
                ShaderPassType::Forward,
                surface_source.source,
                surface_source.entry_point,
            )
            .with_features(surface_source.features),
        )
        .unwrap_or_else(|error| panic!("{label} template should assemble: {error:?}"));

        validate_material_shader_template_wgsl(&assembly.wgsl_source)
            .unwrap_or_else(|error| panic!("{label} composed WGSL should validate: {error:?}"));
    }
}

#[test]
fn render_deferred_gbuffer_template_validates_baked_indirect_output() {
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
    let assembly = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            static_mesh_descriptor(),
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard deferred GBuffer template assembly");

    for expected in [
        "// include: zr_lightmap.wgsl",
        "let baked_indirect = surface.base_color.rgb * diffuse_energy_scale",
        "zr_surface_metallic_diffuse_energy_scale(surface.metallic)",
        "output.emissive = vec4<f32>(output.emissive.rgb + baked_indirect",
    ] {
        assert!(assembly.wgsl_source.contains(expected));
    }
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("deferred GBuffer lightmap WGSL should validate");
}

#[test]
fn standard_pbr_templates_scale_baked_diffuse_by_metallic_once() {
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
    let forward = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh_descriptor(),
            ShaderPassType::Forward,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard forward template assembly");
    let deferred = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            static_mesh_descriptor(),
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard deferred GBuffer template assembly");

    for source in [&forward.wgsl_source, &deferred.wgsl_source] {
        assert!(source.contains("zr_surface_metallic_diffuse_energy_scale(surface.metallic)"));
        assert!(!source.contains("metallic * 0.45"));
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
    assert!(
        !depth_no_alpha
            .wgsl_source
            .contains("surface.normal_ws * 0.5")
    );
    assert!(
        !depth_no_alpha
            .wgsl_source
            .contains("zr_template_gbuffer.wgsl")
    );
    assert!(depth_alpha.wgsl_source.contains("zr_material_surface"));
    assert_include_token!(depth_alpha, "zr_template_depth_alpha.wgsl");
    assert!(
        depth_alpha
            .wgsl_source
            .contains("zr_apply_alpha_clip(surface);")
    );
    assert!(!depth_alpha.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(!depth_alpha.wgsl_source.contains("zr_template_gbuffer.wgsl"));
    assert!(velocity.wgsl_source.contains("fetch_prev_position"));
    assert!(
        velocity
            .wgsl_source
            .contains("struct ZrVelocityVertexInput")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("@location(8) previous_position")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("let previous_input = zr_velocity_vertex_input(v, v.previous_position);")
    );
    assert!(velocity.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn vs_main("));
    assert!(velocity.wgsl_source.contains("fn fs_main("));
    assert!(
        velocity
            .wgsl_source
            .contains("scene.view_proj_unjittered * current_world")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("scene.previous_view_proj_unjittered * previous_world")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("return zr_vs_main_impl(v, instance_index);")
    );
    assert!(!velocity.wgsl_source.contains("zr_material_surface"));
    assert!(velocity_alpha.wgsl_source.contains("zr_material_surface"));
    assert!(
        velocity_alpha
            .wgsl_source
            .contains("zr_surface_fails_alpha_clip(surface)")
    );
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
fn forward_environment_fallback_does_not_synthesize_roughness_without_pmrem() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let sky_reflection = assembly
        .wgsl_source
        .split("fn zr_environment_sky_reflection_color(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_planar_reflection(").next())
        .expect("forward environment source should retain the sky-reflection owner");

    assert!(
        sky_reflection
            .contains("return zr_environment_procedural_sky_color_normalized(reflected);"),
        "a fallback without a PMREM must preserve the reflected direction"
    );
    assert_eq!(
        sky_reflection
            .matches("zr_environment_procedural_sky_color_normalized(")
            .count(),
        1,
        "a fallback without a PMREM must sample the sky only once"
    );
    for forbidden in [
        "zr_environment_procedural_sky_color_normalized(normal)",
        "mix(sharp_reflection, rough_reflection, roughness)",
    ] {
        assert!(
            !sky_reflection.contains(forbidden),
            "a fallback without a PMREM must not synthesize roughness with `{forbidden}`"
        );
    }
}

#[test]
fn forward_environment_skips_zero_weight_probe_samples() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let reflection = assembly
        .wgsl_source
        .split("fn zr_environment_reflection_color_after_planar(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("forward environment source should retain the probe-reflection owner");

    for (weight, target) in [
        ("selection.primary_weight", "selection.primary_index"),
        ("selection.secondary_weight", "selection.secondary_index"),
    ] {
        let gate = format!("if ({weight} > 0.0) {{");
        let guarded_sample = reflection
            .split(&gate)
            .nth(1)
            .unwrap_or_else(|| panic!("the {weight} probe sample should be gated"));
        assert!(
            guarded_sample.contains("zr_environment_probe_color(")
                && guarded_sample.contains(target),
            "the {weight} gate must own its cubemap sample"
        );
    }
}

#[test]
fn forward_environment_skips_all_sampling_when_occlusion_is_zero() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let components = assembly
        .wgsl_source
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("forward environment source should retain the PBR-components owner");

    let occlusion = components
        .find("let clamped_occlusion = clamp(occlusion, 0.0, 1.0);")
        .expect("environment PBR components should clamp occlusion");
    let early_out = components
        .find("if (clamped_occlusion <= 0.0) {")
        .expect("zero occlusion should skip environment sampling");
    let normalization = components
        .find("let normal = zr_environment_normalize_or_zero(normal_ws);")
        .expect("environment PBR components should normalize the normal after its early-out");
    assert!(
        occlusion < early_out && early_out < normalization,
        "zero occlusion must return before normal, PMREM, SH, or BRDF work"
    );
}

#[test]
fn forward_environment_reuses_normalized_normal_for_diffuse_ibl() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let components = assembly
        .wgsl_source
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("forward environment source should retain the PBR-components owner");

    let normalized_normal = components
        .find("let normal = zr_environment_normalize_or_zero(normal_ws);")
        .expect("PBR components should normalize the normal once");
    let diffuse = components
        .find("zr_environment_diffuse_color_normalized(normal)")
        .expect("PBR diffuse IBL should reuse the normalized normal");
    assert!(
        normalized_normal < diffuse,
        "PBR diffuse IBL must consume the already-normalized normal"
    );
    assert!(
        !components.contains("zr_environment_diffuse_color(normal)"),
        "PBR diffuse IBL must not re-enter the defensive normalization wrapper"
    );
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
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_shading_standard_pbr.wgsl")
    );
    assert!(
        assembly
            .wgsl_source
            .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)")
    );
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
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_shading_toon.wgsl")
    );
    assert!(assembly.wgsl_source.contains("ZR_SHADING_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn zr_toon_band"));
    assert!(
        assembly
            .wgsl_source
            .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)")
    );
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
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_gbuffer_encode_toon.wgsl")
    );
    assert!(assembly.wgsl_source.contains("ZR_GBUFFER_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn encode_gbuffer"));
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_template_deferred_gbuffer.wgsl")
    );
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
        advanced_features: StandardPbrMaterialFeatures::default(),
        subsurface_profile_index: 0,
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
