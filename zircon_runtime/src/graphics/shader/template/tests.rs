use std::num::NonZeroU32;

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
fn render_bindless_material_template_requires_a_capacity_and_emits_the_fixed_array_abi() {
    let missing_capacity = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::BINDLESS_MATERIAL)),
    );
    assert_eq!(
        missing_capacity,
        Err(ShaderTemplateAssemblyError::MissingBindlessMaterialSlotCapacity)
    );

    let disabled = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("non-bindless material template");
    let enabled = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_bindless_material_bindings(
                NonZeroU32::new(64).expect("bindless test capacity is non-zero"),
            ),
    )
    .expect("bindless material template");

    assert!(!disabled.wgsl_source.contains("enable wgpu_binding_array;"));
    assert!(!has_include_token(
        &disabled.include_tokens,
        "zr_bindless_material.wgsl"
    ));
    assert!(enabled.wgsl_source.contains("enable wgpu_binding_array;"));
    assert!(
        enabled
            .wgsl_source
            .contains("const ZR_FEATURE_BINDLESS_MATERIAL: bool = true;")
    );
    assert!(
        enabled
            .wgsl_source
            .contains("const ZR_BINDLESS_MATERIAL_SLOT_CAPACITY: u32 = 64u;")
    );
    assert_include_token!(enabled, "zr_bindless_material.wgsl");
}

#[test]
fn standard_pbr_direct_lighting_reuses_per_pixel_material_inputs() {
    let light_loop = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_gpu_light_lighting(")
        .nth(1)
        .and_then(|source| source.split("fn shade_forward(").next())
        .expect("standard PBR should retain the GPU light-grid owner");
    for expected in [
        "let world_normal = zr_normalize_or_zero(surface.normal_ws);",
        "if (surface.shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "let direct_metallic = clamp(surface.metallic, 0.0, 1.0);",
        "direct_f0 = zr_pbr_material_f0(",
        "surface.dielectric_f0,",
        "direct_metallic,",
        "direct_diffuse_brdf = diffuse_color",
        "zr_surface_metallic_diffuse_energy_scale(direct_metallic)",
        "direct_clearcoat_normal: vec3<f32>,",
        "direct_base_energy: vec3<f32>,",
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
        .and_then(|source| source.split("let tile_base =").next())
        .expect("light-grid setup must derive direct PBR material inputs once");
    for expected in [
        "surface.dielectric_f0,",
        "diffuse_color,",
        "direct_metallic,",
    ] {
        assert!(
            direct_material_setup.contains(expected),
            "direct PBR setup must consume the clamped metallic value in `{expected}`"
        );
    }
    assert!(
        !direct_material_setup.contains("surface.metallic"),
        "direct PBR setup must not reuse unbounded surface metallic after normalization"
    );
    assert!(
        !light_loop.contains("zr_pbr_clearcoat_base_energy_scale_normalized("),
        "the light-grid must reuse clearcoat energy prepared by forward shading"
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
        "direct_diffuse_brdf",
    ] {
        assert!(
            standard_lobe.contains(expected),
            "the standard PBR lobe must retain feature-selected anisotropy `{expected}`"
        );
    }
    assert!(
        !standard_lobe.contains("zr_surface_metallic_diffuse_energy_scale("),
        "the Standard-PBR lobe must not recompute source-independent diffuse energy per light"
    );
    let light_dispatch = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_shade_light_vector_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_punctual_light_visibility(")
                .next()
        })
        .expect("standard PBR must retain the light-model dispatch owner");
    let blinn_lobe = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_shade_blinn_phong_light_vector_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_shade_light_vector_normalized(")
                .next()
        })
        .expect("standard PBR must retain the Blinn-Phong lobe owner");
    assert!(light_dispatch.contains("direct_diffuse_brdf: vec3<f32>"));
    assert!(!blinn_lobe.contains("direct_diffuse_brdf"));
    for (feature_guard, material_weight, lobe) in [
        (
            "if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat > 0.0)",
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
            "if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat > 0.0) {",
            "zr_pbr_clearcoat_base_energy_scale_normalized(",
        ),
        (
            "if (specular_transmission > 0.0) {",
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
    assert_eq!(
        forward_shading
            .matches("zr_pbr_clearcoat_base_energy_scale_normalized(")
            .count(),
        1,
        "forward shading must calculate clearcoat base energy only once per pixel"
    );
    let clearcoat_energy = forward_shading
        .find("clearcoat_base_energy = zr_pbr_clearcoat_base_energy_scale_normalized(")
        .expect("forward shading must prepare shared clearcoat base energy");
    let direct_lights = forward_shading
        .find("let direct_lights = zr_standard_pbr_gpu_light_lighting(")
        .expect("forward shading must retain direct lighting");
    let environment_components = forward_shading
        .find("let environment_components =\n        zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized(")
        .expect("forward shading must retain environment lighting");
    let environment_call = forward_shading[environment_components..]
        .split(");")
        .next()
        .expect("forward environment lighting call must have a closing delimiter");
    assert!(
        environment_call.contains("surface.dielectric_f0,"),
        "forward environment lighting must receive the material-derived dielectric F0"
    );
    assert!(
        clearcoat_energy < direct_lights && direct_lights < environment_components,
        "direct and environment lighting must share the prepared clearcoat base energy"
    );
}

#[test]
fn standard_pbr_clearcoat_direct_lighting_uses_its_own_normal_and_skips_zero_normal() {
    let standard_lobe = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_shade_standard_light_vector_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_shade_blinn_phong_light_vector_normalized(")
                .next()
        })
        .expect("standard PBR must retain the direct-light lobe owner");

    let clearcoat_no_l = standard_lobe
        .find("let clearcoat_no_l = max(dot(direct_clearcoat_normal, light_vector), 0.0);")
        .expect("clearcoat direct lighting must use its own normal for NoL");
    let clearcoat_guard = standard_lobe
        .find("if (clearcoat_no_l > 0.0) {")
        .expect("zero clearcoat normal must skip its GGX lobe");
    let clearcoat_lobe = standard_lobe
        .find("clearcoat = zr_clearcoat_lobe(")
        .expect("standard PBR must retain the clearcoat direct-light lobe");

    assert!(
        clearcoat_no_l < clearcoat_guard && clearcoat_guard < clearcoat_lobe,
        "clearcoat NoL must guard GGX evaluation before it can receive a zero normal"
    );
    assert!(
        standard_lobe.contains("* radiance * clearcoat_no_l * clamp(surface.clearcoat, 0.0, 1.0);"),
        "clearcoat direct radiance must use its own clamped material weight and NoL"
    );
    assert!(
        !standard_lobe.contains("* radiance * no_l * surface.clearcoat;"),
        "clearcoat direct radiance must not use the base-layer normal or raw material weight"
    );
}

#[test]
fn standard_pbr_clearcoat_hot_path_reuses_normalized_inputs() {
    let light_loop = STANDARD_PBR_FORWARD_SHADER
        .split("fn zr_standard_pbr_gpu_light_lighting(")
        .nth(1)
        .and_then(|source| source.split("fn shade_forward(").next())
        .expect("standard PBR should retain the GPU light-grid owner");
    assert!(
        !light_loop.contains("zr_pbr_clearcoat_base_energy_scale_normalized("),
        "the GPU light-grid must reuse clearcoat energy from forward shading"
    );

    let forward_shading = STANDARD_PBR_FORWARD_SHADER
        .split("fn shade_forward(")
        .nth(1)
        .expect("standard PBR must retain forward shading");
    let shared_energy = forward_shading
        .split("clearcoat_base_energy = zr_pbr_clearcoat_base_energy_scale_normalized(")
        .nth(1)
        .and_then(|source| source.split(");").next())
        .expect("forward shading must retain its normalized shared base-energy call");
    let surface = shared_energy
        .find("surface,")
        .expect("normalized base energy must receive the surface");
    let clearcoat_normal = shared_energy
        .find("clearcoat_normal,")
        .expect("normalized base energy must receive the prepared clearcoat normal");
    let world_view = shared_energy
        .find("view_dir_ws,")
        .expect("normalized base energy must receive the prepared world view");
    assert!(
        surface < clearcoat_normal && clearcoat_normal < world_view,
        "shared clearcoat energy must not renormalize its prepared normal or view"
    );

    for expected in [
        "clearcoat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);",
        "direct_clearcoat_normal = clearcoat_normal;",
        "zr_pbr_clearcoat_base_energy_scale_normalized(",
        "zr_pbr_advanced_environment_normalized(",
    ] {
        assert!(
            forward_shading.contains(expected),
            "Forward clearcoat hot path must retain `{expected}`"
        );
    }
    assert!(
        !forward_shading.contains("zr_pbr_clearcoat_base_energy_scale(surface, view_dir_ws);"),
        "Forward clearcoat must not call the defensive normalizing wrapper"
    );
    assert!(
        !forward_shading
            .contains("zr_pbr_advanced_environment(surface, ctx.position_ws, view_dir_ws);"),
        "Forward clearcoat must not call the defensive normalizing wrapper"
    );
}

#[test]
fn standard_pbr_clearcoat_scales_ambient_base_layer_once() {
    let forward_shading = STANDARD_PBR_FORWARD_SHADER
        .split("fn shade_forward(")
        .nth(1)
        .expect("standard PBR must retain forward shading");
    assert!(
        forward_shading.contains(
            "diffuse_color * zr_standard_pbr_diffuse_energy_scale(surface) * ambient * clearcoat_base_energy"
        ),
        "clearcoat base energy must attenuate ambient diffuse exactly as it attenuates direct and environment base lighting"
    );
}

macro_rules! assert_missing_include_token {
    ($assembly:expr, $token:expr) => {
        assert!(!has_include_token(&($assembly).include_tokens, $token));
    };
}

#[path = "tests/environment.rs"]
mod environment;
#[path = "tests/environment_only_pbr.rs"]
mod environment_only_pbr;
#[path = "tests/environment_specular_occlusion.rs"]
mod environment_specular_occlusion;
#[path = "tests/standard_material_surface_template.rs"]
mod standard_material_surface_template;
#[path = "tests/standard_pbr_specialization.rs"]
mod standard_pbr_specialization;
#[path = "tests/surface_modules.rs"]
mod surface_modules;

#[path = "tests/material_template_assembly.rs"]
mod material_template_assembly;
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
        normal_scale: 1.0,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        metallic_roughness_texture_transform: RenderMaterialTextureTransform::default(),
        metallic_roughness_texture_uv_channel: 0,
        occlusion_texture: None,
        occlusion_texture_transform: RenderMaterialTextureTransform::default(),
        occlusion_texture_uv_channel: 0,
        occlusion_strength: 1.0,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        emissive_texture_transform: RenderMaterialTextureTransform::default(),
        emissive_texture_uv_channel: 0,
        clearcoat_normal_texture_transform: RenderMaterialTextureTransform::default(),
        clearcoat_normal_texture_uv_channel: 0,
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
        separate_translucency: false,
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
