use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_ID_BLINN_PHONG,
    SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_ID_UNLIT,
};
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;

use super::*;

#[test]
fn builtin_deferred_include_tokens_accept_manifest_and_wgsl_forms() {
    for token in [
        "zr_shade_deferred_unlit",
        "zr_shade_deferred_unlit.wgsl",
        "zr_shade_deferred_blinn_phong",
        "zr_shade_deferred_standard_pbr.wgsl",
    ] {
        assert!(builtin_deferred_include_token(token));
    }
}

#[test]
fn deferred_shading_function_names_strip_include_prefix_and_extension() {
    assert_eq!(
        deferred_shading_function_name("zr_shade_deferred_toon.wgsl"),
        "shade_deferred_toon"
    );
}

#[test]
fn builtin_shading_model_ids_are_reserved_by_static_dispatch() {
    assert_eq!(SHADING_MODEL_ID_UNLIT.value(), 0);
    assert_eq!(SHADING_MODEL_ID_BLINN_PHONG.value(), 1);
    assert_eq!(SHADING_MODEL_ID_STANDARD_PBR.value(), 2);
}

#[test]
fn fragment_template_keeps_the_fullscreen_vertex_entry_in_its_dedicated_module() {
    assert!(
        !DEFERRED_LIGHTING_TEMPLATE.contains("@vertex"),
        "the deferred fragment module must not retain the separately compiled fullscreen vertex entry"
    );
    assert!(!DEFERRED_LIGHTING_TEMPLATE.contains("fn vs_main("));
}

#[test]
fn deferred_source_assembly_constructs_only_the_requested_module_closure() {
    let source = include_str!("../shader_source.rs");
    let body = source
        .split_once("pub(in crate::graphics::scene::scene_renderer::deferred) fn assemble_deferred_lighting_shader_source")
        .expect("deferred source assembly function must exist")
        .1
        .split_once("\nfn deferred_module_resolution_error")
        .expect("deferred source assembly function must have a stable end")
        .0;

    assert!(body.contains("with_builtin_modules_for_roots("));
    assert!(!body.contains("with_builtin_modules();"));
    assert!(body.contains("roots.iter().any(|root| root == VOLUMETRIC_INCLUDE_TOKEN)"));
}

#[test]
fn standard_pbr_preview_assembles_only_the_standard_builtin_material_variant() {
    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::StandardPbrPreview,
        ),
    )
    .expect("standard PBR preview source should assemble");

    assert!(source.contains("// include: zr_shade_deferred_standard_pbr.wgsl"));
    assert!(source.contains("// include: zr_pbr_extras.wgsl"));
    assert!(source.contains("fn zr_pbr_isotropic_ggx("));
    assert!(!source.contains("// include: zr_shade_deferred_blinn_phong.wgsl"));
    assert!(!source.contains("// include: zr_shade_deferred_unlit.wgsl"));
    assert!(!source.contains("// include: zr_shade_deferred_subsurface.wgsl"));
    assert!(!source.contains("shade_deferred_blinn_phong("));
    assert!(!source.contains("shade_deferred_unlit("));
}

#[test]
fn standard_pbr_preview_uses_source_independent_diffuse_and_shared_ggx_specular() {
    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::StandardPbrPreview,
        ),
    )
    .expect("standard PBR preview source should assemble");

    for required in [
        "fn zr_pbr_isotropic_ggx(",
        "let specular = zr_pbr_isotropic_ggx(",
        "direct_diffuse_brdf * radiance * lambert",
        "radiance * specular * lambert",
        "zr_surface_metallic_diffuse_energy_scale(direct_metallic)",
    ] {
        assert!(
            source.contains(required),
            "deferred Standard PBR must retain source-independent diffuse/GGX contract `{required}`"
        );
    }
    for rejected in [
        "struct ZrPbrSpecularComponents",
        "fn zr_pbr_isotropic_ggx_components(",
        "specular_components.fresnel",
    ] {
        assert!(!source.contains(rejected));
    }
    assert!(
        !source.contains("zr_pbr_diffuse_energy_scale("),
        "standard PBR preview must use the shared metallic diffuse-energy owner"
    );
}

#[test]
fn standard_pbr_preview_prunes_generic_environment_api_but_keeps_local_reflections() {
    let standard = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::StandardPbrPreview,
        ),
    )
    .expect("standard PBR preview source should assemble");
    let full_scene = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new()
            .with_deferred_lighting_profile(SceneRendererDeferredLightingProfile::FullScene),
    )
    .expect("full-scene deferred source should assemble");

    for required in [
        "@group(1) @binding(16)",
        "@group(1) @binding(17)",
        "@group(1) @binding(18)",
        "@group(1) @binding(29)",
        "@group(1) @binding(30)",
        "fn zr_environment_select_probes(",
        "fn zr_environment_planar_reflection(",
        "fn zr_environment_pbr_indirect(",
    ] {
        assert!(
            standard.contains(required),
            "standard PBR preview must retain local reflection source `{required}`"
        );
    }
    for excluded in [
        "fn zr_environment_fix_source_cube_lookup(",
        "fn zr_environment_source_cube_color_at_lod(",
        "fn zr_environment_specular_pmrem_color_at_lod(",
        "fn zr_environment_env_brdf_approx(",
        "fn zr_environment_sh9_eval(",
        "fn zr_environment_irradiance_cube_color(",
        "fn zr_environment_procedural_sky_color(",
        "fn zr_environment_sky_color(",
        "fn zr_environment_diffuse_color(",
    ] {
        assert!(
            !standard.contains(excluded),
            "standard PBR preview must prune unreachable API `{excluded}`"
        );
        assert!(
            full_scene.contains(excluded),
            "full-scene deferred must retain generic API `{excluded}`"
        );
    }
    assert!(
        standard.len() < full_scene.len(),
        "specialized preview should compile less source, standard={} full-scene={}",
        standard.len(),
        full_scene.len(),
    );
}

#[test]
fn preview_profiles_reject_custom_shading_models_before_source_export() {
    let descriptor = ShadingModelDescriptor::new(
        ShadingModelId::new(128),
        "toon",
        "package://toon/forward.wgsl",
        "package://toon/gbuffer.wgsl",
        "package://toon/deferred.wgsl",
        GBufferChannelMask::standard_deferred_v1(),
    );

    for profile in [
        SceneRendererDeferredLightingProfile::StandardPbrPreview,
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
    ] {
        let error = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new()
                .with_deferred_lighting_profile(profile)
                .with_shading_model_descriptor(descriptor.clone()),
        )
        .expect_err("preview profiles must reject custom shading models");

        assert_eq!(
            error,
            DeferredLightingShaderSourceError::CustomShadingModelsUnsupportedByProfile { profile }
        );
    }
}

#[test]
fn environment_only_pbr_preview_assembles_ibl_without_direct_lighting_dependencies() {
    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        ),
    )
    .expect("environment-only PBR preview source should assemble");

    assert!(source.contains("// include: zr_environment.wgsl"));
    assert!(source.contains("zr_environment_pbr_indirect("));
    assert!(source.contains("fn zr_environment_pbr_indirect_with_dielectric_f0_normalized("));
    assert!(source.contains(
        "let environment_lights = zr_environment_pbr_indirect_with_dielectric_f0_normalized("
    ));
    assert!(
        !source.contains("zr_pbr_diffuse_energy_scale("),
        "environment-only deferred source must use the shared metallic diffuse-energy owner"
    );
    assert!(!source.contains("// include: zr_gpu_scene.wgsl"));
    assert!(!source.contains("// include: zr_light_grid.wgsl"));
    assert!(!source.contains("// include: zr_light_cookie.wgsl"));
    assert!(!source.contains("// include: zr_lightmap.wgsl"));
    assert!(!source.contains("// include: zr_shadow.wgsl"));
    assert!(!source.contains("// include: zr_volumetric.wgsl"));
    assert!(!source.contains("// include: zr_pbr_extras.wgsl"));
    assert!(!source.contains("fn zr_pbr_isotropic_ggx("));
    assert!(!source.contains("fn fs_main_sss("));
}

#[test]
fn environment_only_pbr_preview_mirrors_rotation_abi() {
    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        ),
    )
    .expect("environment-only PBR source should assemble");

    assert!(source.contains("fn zr_environment_pbr_indirect("));
    let scene_uniform = source
        .split("struct SceneUniform {")
        .nth(1)
        .and_then(|source| source.split("};").next())
        .expect("deferred preview must declare SceneUniform");
    assert!(
        scene_uniform.contains("environment_rotation_sin_cos: vec4<f32>,"),
        "deferred preview SceneUniform must mirror the rotation tail"
    );
    assert!(
        scene_uniform
            .find("environment_sample_params")
            .expect("sample params")
            < scene_uniform
                .find("environment_rotation_sin_cos")
                .expect("rotation tail"),
        "the rotation field must append after existing SceneUniform fields"
    );
}

#[test]
fn environment_only_pbr_preview_retains_generic_environment_for_provider_upgrades() {
    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        ),
    )
    .expect("environment-only PBR source should assemble");

    for required in [
        "@group(1) @binding(16)",
        "@group(1) @binding(17)",
        "@group(1) @binding(18)",
        "@group(1) @binding(29)",
        "@group(1) @binding(30)",
        "fn zr_environment_select_probes(",
        "fn zr_environment_planar_reflection(",
        "fn zr_environment_specular_pmrem_color_at_lod(",
        "fn zr_environment_irradiance_cube_color(",
    ] {
        assert!(
            source.contains(required),
            "environment-only deferred must retain provider-upgrade source `{required}`"
        );
    }
}

#[test]
fn deferred_pbr_gbuffer_normal_decode_uses_zero_safe_normalization() {
    let environment_only = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        ),
    )
    .expect("environment-only PBR source should assemble");

    for (label, source, expected_normal_decode) in [
        (
            "generic",
            DEFERRED_LIGHTING_SHADER,
            "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));",
        ),
        (
            "environment-only",
            environment_only.as_str(),
            "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));",
        ),
    ] {
        assert!(
            source.contains("fn normalize_or_zero(value: vec3<f32>) -> vec3<f32>"),
            "{label} deferred source must retain the zero-safe normal helper"
        );
        assert!(
            source.contains(expected_normal_decode),
            "{label} deferred source must safely decode a degenerate GBuffer normal"
        );
        assert!(
            !source.contains("let normal = normalize(encoded_normal"),
            "{label} deferred source must not normalize a potentially zero GBuffer normal directly"
        );
    }

    assert!(
        DEFERRED_LIGHTING_SHADER.contains("fn fs_main_sss("),
        "generic deferred source must retain its SSS fragment entry point"
    );
    assert!(
        DEFERRED_LIGHTING_SHADER
            .contains("let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));"),
        "generic SSS deferred source must safely decode a degenerate GBuffer normal"
    );
}
