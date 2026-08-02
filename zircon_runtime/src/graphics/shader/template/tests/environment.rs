use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::{
    assemble_material_shader_template, material_template_request,
    standard_material_surface_source_for_features, static_mesh_descriptor,
    MaterialShaderTemplateRequest,
};

fn environment_pbr_composition_source(source: &str) -> &str {
    source
        .split("fn zr_environment_pbr_components_from_reflection(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_components(").next())
        .expect("forward environment source should retain shared PBR composition")
}

#[test]
fn builtin_standard_forward_prunes_generic_environment_api_but_keeps_local_reflections() {
    let surface = standard_material_surface_source_for_features(ShaderFeatureBits::default(), 0.5);
    let standard = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh_descriptor(),
            ShaderPassType::Forward,
            surface.source,
            surface.entry_point,
        )
        .with_features(surface.features),
    )
    .expect("builtin Standard-PBR Forward template assembly");
    let custom = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("custom Forward template assembly");

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
            standard.wgsl_source.contains(required),
            "builtin Standard-PBR Forward must retain local reflection source `{required}`"
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
            !standard.wgsl_source.contains(excluded),
            "builtin Standard-PBR Forward must prune unreachable API `{excluded}`"
        );
        assert!(
            custom.wgsl_source.contains(excluded),
            "custom Forward must retain generic API `{excluded}`"
        );
    }
    let environment_hash = |assembly: &super::super::assemble::MaterialShaderTemplateAssembly| {
        assembly
            .include_tokens
            .iter()
            .zip(&assembly.include_content_hashes)
            .find_map(|(token, hash)| (token == "zr_environment.wgsl").then_some(hash.as_str()))
            .expect("Forward assembly should retain the canonical environment token")
    };
    assert_ne!(
        environment_hash(&standard),
        environment_hash(&custom),
        "builtin and custom Forward assemblies must separate environment cache content"
    );
    assert!(
        standard.wgsl_source.len() < custom.wgsl_source.len(),
        "pruned Standard-PBR Forward should compile less source, standard={} custom={}",
        standard.wgsl_source.len(),
        custom.wgsl_source.len(),
    );
}

#[test]
fn forward_environment_rotation_uses_cpu_precomputed_trigonometry() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let rotation = assembly
        .wgsl_source
        .split("fn zr_environment_rotated_direction(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_fix_cube_lookup_for_face_size(")
                .next()
        })
        .expect("forward environment source should retain the rotation helper");

    assert!(
        rotation.contains("scene.environment_rotation_sin_cos.z < 0.5"),
        "zero rotation should retain its no-op fast path"
    );
    assert!(
        rotation.contains("scene.environment_rotation_sin_cos.x")
            && rotation.contains("scene.environment_rotation_sin_cos.y"),
        "environment rotation must consume CPU-precomputed sine and cosine"
    );
    assert!(
        !rotation.contains("sin(rotation)") && !rotation.contains("cos(rotation)"),
        "environment rotation must not execute trigonometry per pixel"
    );
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
fn forward_environment_rejects_zero_normal_or_view_before_texture_work() {
    for (label, request) in [
        (
            "generic",
            material_template_request(static_mesh_descriptor(), ShaderPassType::Forward),
        ),
        (
            "environment-only",
            material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
                .with_features(ShaderFeatureBits::new(
                    ShaderFeatureBits::ENVIRONMENT_ONLY_PBR,
                )),
        ),
    ] {
        let assembly = assemble_material_shader_template(request)
            .unwrap_or_else(|error| panic!("{label} Forward template assembly: {error}"));
        let components = assembly
            .wgsl_source
            .split("fn zr_environment_pbr_components(")
            .nth(1)
            .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
            .unwrap_or_else(|| panic!("{label} Forward should retain PBR components"));

        let normal = components
            .find("let normal = zr_environment_normalize_or_zero(normal_ws);")
            .unwrap_or_else(|| panic!("{label} PBR should normalize the normal"));
        let view = components
            .find("let view_dir = zr_environment_normalize_or_zero(view_dir_ws);")
            .unwrap_or_else(|| panic!("{label} PBR should normalize the view direction"));
        let invalid_direction = components
            .find("if (all(normal == vec3<f32>(0.0)) || all(view_dir == vec3<f32>(0.0))) {")
            .unwrap_or_else(|| panic!("{label} PBR should reject zero cube-sampling directions"));
        let reflection = components
            .find("let reflection =")
            .unwrap_or_else(|| panic!("{label} PBR should resolve reflection after validation"));
        assert!(
            normal < view && view < invalid_direction && invalid_direction < reflection,
            "{label} PBR must reject zero normal/view inputs before PMREM, SH, or BRDF work"
        );
        assert!(
            components[invalid_direction..reflection]
                .contains("return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));"),
            "{label} PBR must return before resolving reflection sampling"
        );
    }
}

#[test]
fn forward_clearcoat_environment_rejects_zero_direction_before_texture_work() {
    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::PBR_CLEARCOAT)),
    )
    .expect("clearcoat Forward template assembly");
    let clearcoat_environment = assembly
        .wgsl_source
        .split("fn zr_pbr_advanced_environment(")
        .nth(1)
        .and_then(|source| source.split("fn zr_pbr_viewport_uv(").next())
        .expect("clearcoat Forward should retain its advanced environment owner");

    let coat_normal = clearcoat_environment
        .find("let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);")
        .expect("clearcoat environment should normalize its normal");
    let view_direction = clearcoat_environment
        .find("let normalized_view_dir = zr_normalize_or_zero(view_dir);")
        .expect("clearcoat environment should normalize its view direction");
    let invalid_direction = clearcoat_environment
        .find(
            "if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {",
        )
        .expect("clearcoat environment should reject zero reflection directions");
    let reflection = clearcoat_environment
        .find("let reflected = zr_environment_reflection_color(")
        .expect("clearcoat environment should resolve reflection after validation");

    assert!(
        coat_normal < view_direction
            && view_direction < invalid_direction
            && invalid_direction < reflection,
        "clearcoat must reject zero normal/view inputs before planar, cubemap, or BRDF work"
    );
    assert!(
        clearcoat_environment[invalid_direction..reflection].contains("return vec3<f32>(0.0);"),
        "clearcoat must return before reflection sampling"
    );
}

#[test]
fn forward_clearcoat_zero_direction_preserves_base_layer_energy() {
    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::PBR_CLEARCOAT)),
    )
    .expect("clearcoat Forward template assembly");
    let clearcoat_energy = assembly
        .wgsl_source
        .split("fn zr_pbr_clearcoat_base_energy_scale(")
        .nth(1)
        .and_then(|source| source.split("fn zr_pbr_advanced_environment(").next())
        .expect("clearcoat Forward should retain its base-energy owner");

    let coat_normal = clearcoat_energy
        .find("let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);")
        .expect("clearcoat base energy should normalize its normal");
    let view_direction = clearcoat_energy
        .find("let normalized_view_dir = zr_normalize_or_zero(view_dir);")
        .expect("clearcoat base energy should normalize its view direction");
    let invalid_direction = clearcoat_energy
        .find(
            "if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {",
        )
        .expect("clearcoat base energy should reject invalid directions");
    let no_v = clearcoat_energy
        .find("let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);")
        .expect("clearcoat base energy should use normalized directions");

    assert!(
        coat_normal < view_direction
            && view_direction < invalid_direction
            && invalid_direction < no_v,
        "clearcoat base energy must preserve the base layer before invalid-direction BRDF work"
    );
    assert!(
        clearcoat_energy[invalid_direction..no_v].contains("return vec3<f32>(1.0);"),
        "invalid clearcoat directions must not attenuate the base layer"
    );
}

#[test]
fn forward_environment_keeps_local_reflections_when_global_source_is_disabled() {
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
    let reflection = assembly
        .wgsl_source
        .split("fn zr_environment_reflection_color_after_planar(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_diffuse_color_normalized(")
                .next()
        })
        .expect("forward environment source should retain reflection fallback mixing");

    assert!(
        !components.contains("if (!zr_environment_is_enabled() || !is_standard_pbr)"),
        "the global sky/source flag must not disable independent local reflection providers"
    );
    let material_gate = components
        .find("if (!is_standard_pbr) {")
        .expect("only the material model should unconditionally disable PBR indirect lighting");
    let global_availability = components
        .find(
            "let has_global_environment = zr_environment_is_enabled()\n        && environment_intensity > 0.0;",
        )
        .expect("PBR components should combine the global source flag and intensity");
    let provider_early_out = components
        .find(
            "if (!has_global_environment\n        && zr_env_probe_header.probe_count == 0u\n        && zr_env_planar.sample_params.w < 0.5)",
        )
        .expect("PBR components should return only when every environment provider is unavailable");
    assert!(
        material_gate < global_availability && global_availability < provider_early_out,
        "global availability must be resolved before the provider-aware early-out"
    );
    assert!(reflection.contains(
        "let has_global_environment = zr_environment_is_enabled()\n        && scene.environment_params.y > 0.0;"
    ));
}

#[test]
fn forward_environment_keeps_local_provider_intensity_independent_from_global_intensity() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let probe = assembly
        .wgsl_source
        .split("fn zr_environment_probe_color(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_sky_reflection_color(")
                .next()
        })
        .expect("forward environment source should retain probe sampling");
    let indirect = assembly
        .wgsl_source
        .split("fn zr_environment_pbr_indirect(")
        .nth(1)
        .expect("forward environment source should retain PBR indirect composition");

    assert!(
        probe.contains(").rgb * max(probe.misc.x, 0.0);"),
        "reflection probes must retain their own non-negative intensity"
    );
    assert!(
        indirect.contains("return components.diffuse + components.specular;"),
        "PBR indirect must return the provider-composed components directly"
    );
    assert!(
        !indirect.contains("scene.environment_params.y"),
        "global environment intensity must not be reapplied after local provider composition"
    );
}

#[test]
fn forward_environment_skips_global_sampling_when_intensity_is_not_positive_and_local_reflections_are_absent(
) {
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

    let intensity = components
        .find("let environment_intensity = max(scene.environment_params.y, 0.0);")
        .expect("environment PBR components should clamp intensity");
    let global_availability = components
        .find(
            "let has_global_environment = zr_environment_is_enabled()\n        && environment_intensity > 0.0;",
        )
        .expect("environment PBR components should combine source availability and intensity");
    let early_out = components
        .find(
            "if (!has_global_environment\n        && zr_env_probe_header.probe_count == 0u\n        && zr_env_planar.sample_params.w < 0.5)",
        )
        .expect("zero intensity should skip global environment sampling only without local reflections");
    let normalization = components
        .find("let normal = zr_environment_normalize_or_zero(normal_ws);")
        .expect("environment PBR components should normalize the normal after its early-out");
    assert!(
        intensity < global_availability
            && global_availability < early_out
            && early_out < normalization,
        "non-positive intensity without local providers must return before normal, PMREM, SH, or BRDF work"
    );
}

#[test]
fn forward_environment_skips_global_ibl_samples_when_only_local_reflections_are_active() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let reflection = assembly
        .wgsl_source
        .split("fn zr_environment_reflection_color_after_planar(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_diffuse_color_normalized(")
                .next()
        })
        .expect("forward environment source should retain reflection fallback mixing");
    let components = environment_pbr_composition_source(&assembly.wgsl_source);

    let global_availability = reflection
        .find(
            "let has_global_environment = zr_environment_is_enabled()\n        && scene.environment_params.y > 0.0;",
        )
        .expect("reflection mixing should combine the global source flag and intensity");
    let first_sky_sample = reflection
        .find("zr_environment_sky_reflection_color(reflected, clamped_roughness)")
        .expect("reflection mixing should retain the global sky fallback");
    assert!(
        global_availability < first_sky_sample,
        "global availability must be resolved before any sky or PMREM sample"
    );
    assert!(reflection.contains(
        "if (zr_env_probe_header.probe_count == 0u) {\n        if (!has_global_environment) {\n            return vec3<f32>(0.0);"
    ));
    assert!(reflection.contains("if (sky_weight > 0.0 && has_global_environment) {"));
    assert!(components.contains(
        "if (diffuse_energy_scale > 0.0\n        && has_global_environment\n        && any(diffuse_color != vec3<f32>(0.0)))"
    ));
}

#[test]
fn forward_environment_skips_texture_work_for_zero_diffuse_and_reflection_contributions() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let provider_components = assembly
        .wgsl_source
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("forward environment source should retain the PBR-components owner");
    let components = environment_pbr_composition_source(&assembly.wgsl_source);

    let diffuse_sample = "zr_environment_diffuse_color_normalized(normal)";
    let diffuse_branch = components
        .split(
            "if (diffuse_energy_scale > 0.0\n        && has_global_environment\n        && any(diffuse_color != vec3<f32>(0.0)))\n    {",
        )
        .nth(1)
        .and_then(|source| source.split("\n    }\n    let reflection =").next())
        .expect("black diffuse input should gate the irradiance-sampling branch");
    assert!(
        diffuse_branch.contains(diffuse_sample),
        "the black-diffuse gate must own the irradiance sample"
    );
    assert_eq!(
        components.matches(diffuse_sample).count(),
        1,
        "PBR components should contain one gated irradiance sample"
    );

    let reflection = provider_components
        .find("let reflection = zr_environment_reflection_color_normalized(")
        .expect("PBR components should retain environment reflection resolution");
    let zero_specular = components
        .find("var specular_environment = vec3<f32>(0.0);")
        .expect("PBR components should default to zero specular environment");
    let reflection_zero_gate = components
        .find("if (any(reflection != vec3<f32>(0.0))) {")
        .expect("zero reflection radiance should skip the BRDF LUT sample");
    let brdf_sample = "reflection * zr_environment_env_brdf_lut(f0, clamped_roughness, no_v)";
    let f0 = "let f0 = mix(";
    let no_v = "let no_v = clamp(dot(normal, view_dir), 0.0, 1.0);";
    let reflection_branch = components
        .split("if (any(reflection != vec3<f32>(0.0))) {")
        .nth(1)
        .and_then(|source| {
            source
                .split("\n    }\n    return ZrEnvironmentPbrComponents(")
                .next()
        })
        .expect("zero reflection radiance should gate the split-sum branch");
    assert!(
        reflection
            < provider_components
                .find("return zr_environment_pbr_components_from_reflection(")
                .expect("provider reflection should feed shared PBR composition"),
        "the provider must resolve reflection before shared PBR composition"
    );
    assert!(
        zero_specular < reflection_zero_gate,
        "the zero-reflection gate must follow the zero default"
    );
    assert!(
        reflection_branch.contains(f0)
            && reflection_branch.contains(no_v)
            && reflection_branch.contains(brdf_sample),
        "the zero-reflection gate must own F0, NdotV, and the split-sum BRDF lookup"
    );
    assert_eq!(components.matches(f0).count(), 1);
    assert_eq!(components.matches(no_v).count(), 1);
    assert_eq!(
        components.matches(brdf_sample).count(),
        1,
        "PBR components should contain one gated split-sum BRDF lookup"
    );
}

#[test]
fn forward_environment_reuses_normalized_normal_for_diffuse_ibl() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let provider_components = assembly
        .wgsl_source
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("forward environment source should retain the PBR-components owner");
    let components = environment_pbr_composition_source(&assembly.wgsl_source);

    let normalized_normal = provider_components
        .find("let normal = zr_environment_normalize_or_zero(normal_ws);")
        .expect("PBR components should normalize the normal once");
    let diffuse = components
        .find("zr_environment_diffuse_color_normalized(normal)")
        .expect("PBR diffuse IBL should reuse the normalized normal");
    assert!(
        normalized_normal
            < provider_components
                .find("return zr_environment_pbr_components_from_reflection(")
                .expect("normalized inputs should be passed to shared PBR composition"),
        "PBR composition must receive the already-normalized normal"
    );
    assert!(
        !components.contains("zr_environment_diffuse_color(normal)"),
        "PBR diffuse IBL must not re-enter the defensive normalization wrapper"
    );
}

#[test]
fn environment_only_forward_specialization_removes_unreachable_lighting_modules() {
    let generic = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("generic Forward template assembly");
    let specialized = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward).with_features(
            ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        ),
    )
    .expect("environment-only Forward template assembly");

    assert!(specialized
        .wgsl_source
        .contains("const ZR_FEATURE_ENVIRONMENT_ONLY_PBR: bool = true;"));
    assert!(specialized
        .wgsl_source
        .contains("zr_environment_pbr_indirect("));
    for forbidden in [
        "zr_standard_pbr_gpu_light_lighting(",
        "zr_light_cookie_factor(",
        "zr_lightmap_baked_irradiance(",
        "zr_gpu_light_shadow_visibility(",
        "zr_volumetric_apply(",
    ] {
        assert!(
            !specialized.wgsl_source.contains(forbidden),
            "environment-only Forward must not retain unreachable `{forbidden}`"
        );
    }
    for forbidden_token in [
        "zr_light_cookie.wgsl",
        "zr_irradiance_volume.wgsl",
        "zr_lightmap.wgsl",
        "zr_light_grid.wgsl",
        "zr_shadow.wgsl",
        "zr_volumetric.wgsl",
        "zr_pbr_extras.wgsl",
    ] {
        assert!(
            !specialized
                .include_tokens
                .iter()
                .any(|token| token == forbidden_token),
            "environment-only Forward must not hash unreachable `{forbidden_token}`"
        );
    }
    assert!(
        specialized.wgsl_source.len() * 4 <= generic.wgsl_source.len() * 3,
        "environment-only Forward should remove at least 25% of assembled WGSL, generic={} specialized={}",
        generic.wgsl_source.len(),
        specialized.wgsl_source.len(),
    );
}

#[test]
fn environment_only_forward_specialization_excludes_unreachable_environment_api() {
    let generic = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("generic Forward template assembly");
    let specialized = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward).with_features(
            ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        ),
    )
    .expect("environment-only Forward template assembly");

    for required in [
        "zr_environment_sky_reflection_color(",
        "zr_environment_diffuse_color_normalized(",
        "zr_environment_env_brdf_lut(",
    ] {
        assert!(
            specialized.wgsl_source.contains(required),
            "environment-only Forward must retain global IBL `{required}`"
        );
    }
    for excluded_source in [
        "@group(1) @binding(16)",
        "@group(1) @binding(17)",
        "@group(1) @binding(18)",
        "@group(1) @binding(29)",
        "@group(1) @binding(30)",
        "fn zr_environment_select_probes(",
        "fn zr_environment_probe_color(",
        "fn zr_environment_planar_reflection(",
        "fn zr_environment_reflection_color_after_planar(",
        "fn zr_environment_reflection_color(",
        "fn zr_environment_reflection_color_normalized(",
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
            !specialized.wgsl_source.contains(excluded_source),
            "environment-only Forward must exclude unreachable source `{excluded_source}`"
        );
        assert!(
            generic.wgsl_source.contains(excluded_source),
            "generic Forward must retain `{excluded_source}`"
        );
    }
    let generic_environment_hash = generic
        .include_tokens
        .iter()
        .zip(&generic.include_content_hashes)
        .find_map(|(token, hash)| (token == "zr_environment.wgsl").then_some(hash))
        .expect("generic Forward should retain the canonical environment include token");
    let specialized_environment_hash = specialized
        .include_tokens
        .iter()
        .zip(&specialized.include_content_hashes)
        .find_map(|(token, hash)| (token == "zr_environment.wgsl").then_some(hash))
        .expect("environment-only Forward should retain the canonical environment include token");
    assert_ne!(
        generic_environment_hash, specialized_environment_hash,
        "generic and environment-only composites must have distinct environment content hashes"
    );
    assert!(
        specialized.wgsl_source.len() * 2 <= generic.wgsl_source.len(),
        "global-only IBL specialization should remove at least half of comparable WGSL, generic={} specialized={}",
        generic.wgsl_source.len(),
        specialized.wgsl_source.len(),
    );
}
