use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::{
    assemble_material_shader_template, material_template_request,
    standard_material_surface_source_for_features, static_mesh_descriptor,
    MaterialShaderTemplateRequest,
};

fn environment_pbr_composition_source(source: &str) -> String {
    wgsl_function_source(source, "fn zr_environment_pbr_components_from_reflection(")
}

fn wgsl_line_break_len_at(bytes: &[u8], index: usize) -> Option<usize> {
    match bytes.get(index) {
        Some(b'\n' | b'\x0B' | b'\x0C') => Some(1),
        Some(b'\r') => Some(if bytes.get(index + 1) == Some(&b'\n') {
            2
        } else {
            1
        }),
        Some(&0xC2) if bytes.get(index + 1) == Some(&0x85) => Some(2),
        Some(&0xE2)
            if bytes.get(index + 1) == Some(&0x80)
                && matches!(bytes.get(index + 2), Some(0xA8 | 0xA9)) =>
        {
            Some(3)
        }
        _ => None,
    }
}

pub(super) fn wgsl_without_comments(source: &str) -> String {
    let mut code = String::with_capacity(source.len());
    let mut index = 0;
    while index < source.len() {
        let remaining = &source[index..];
        if remaining.starts_with("//") {
            index += 2;
            while index < source.len() && wgsl_line_break_len_at(source.as_bytes(), index).is_none()
            {
                index += source[index..]
                    .chars()
                    .next()
                    .expect("source index must remain on a UTF-8 boundary")
                    .len_utf8();
            }
            code.push(' ');
            continue;
        }
        if remaining.starts_with("/*") {
            let mut depth = 1usize;
            index += 2;
            while depth > 0 {
                let comment = source
                    .get(index..)
                    .expect("WGSL block comment must be terminated");
                if comment.starts_with("/*") {
                    depth += 1;
                    index += 2;
                } else if comment.starts_with("*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += comment
                        .chars()
                        .next()
                        .expect("WGSL block comment must be terminated")
                        .len_utf8();
                }
            }
            code.push(' ');
            continue;
        }
        let character = remaining
            .chars()
            .next()
            .expect("source index must remain on a UTF-8 boundary");
        code.push(character);
        index += character.len_utf8();
    }
    code
}

pub(super) fn wgsl_function_source(source: &str, signature: &str) -> String {
    let code = wgsl_without_comments(source);
    let start = code
        .find(signature)
        .unwrap_or_else(|| panic!("missing WGSL function `{signature}`"));
    let body_start = code[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing opening brace for `{signature}`"));
    let mut depth = 0usize;
    for (offset, byte) in code.as_bytes()[body_start..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("unbalanced braces for `{signature}`"));
                if depth == 0 {
                    return code[start..=body_start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("missing closing brace for `{signature}`");
}

#[test]
fn wgsl_function_source_ignores_commented_out_tokens_and_nested_braces() {
    let source = r#"
        /* fn shade_forward() { let commented = true; } */
        fn shade_forward() {
            // let commented = true;
            if (true) { /* { nested comment } */ let actual = true; }
        }
    "#;

    let function = wgsl_function_source(source, "fn shade_forward(");

    assert!(function.contains("let actual = true;"));
    assert!(!function.contains("let commented = true;"));
}

#[test]
fn environment_pbr_composition_source_ignores_commented_and_following_functions() {
    let source = r#"
        /* fn zr_environment_pbr_components_from_reflection() {
            let commented = true;
        } */
        fn zr_environment_pbr_components_from_reflection() {
            let actual = true;
        }
        fn zr_environment_pbr_components() {
            let following = true;
        }
    "#;

    let function = environment_pbr_composition_source(source);

    assert!(function.contains("let actual = true;"));
    assert!(!function.contains("let commented = true;"));
    assert!(!function.contains("let following = true;"));
}

#[test]
fn wgsl_function_source_ends_line_comments_at_every_wgsl_line_break() {
    for line_break in [
        "\n", "\r", "\r\n", "\u{000B}", "\u{000C}", "\u{0085}", "\u{2028}", "\u{2029}",
    ] {
        let source = format!(
            "fn shade_forward() {{ // let commented = true;{line_break}let actual = true; }}"
        );

        let function = wgsl_function_source(&source, "fn shade_forward(");

        assert!(function.contains("let actual = true;"));
        assert!(!function.contains("let commented = true;"));
    }
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
    let standard_source = wgsl_without_comments(&standard.wgsl_source);
    let custom_source = wgsl_without_comments(&custom.wgsl_source);

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
            standard_source.contains(required),
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
            !standard_source.contains(excluded),
            "builtin Standard-PBR Forward must prune unreachable API `{excluded}`"
        );
        assert!(
            custom_source.contains(excluded),
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
    let rotation = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_rotated_direction(",
    );

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
    let sky_reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_sky_reflection_color(",
    );

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
fn forward_environment_skips_zero_intensity_probes_before_spatial_weighting() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let selection = wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_select_probes(");

    let intensity_gate = selection
        .find("if (!(probe.misc.x > 0.0)) {")
        .expect("GPU probe selection must reject a disabled probe before selecting blend weights");
    let spatial_weight = selection
        .find("let weight = zr_environment_probe_weight(probe, world_position);")
        .expect("GPU probe selection must retain spatial blend weighting for active probes");

    assert!(
        intensity_gate < spatial_weight,
        "a zero-intensity probe must not consume blend weight or trigger cubemap sampling"
    );
    assert_eq!(
        selection[intensity_gate..spatial_weight].trim(),
        "if (!(probe.misc.x > 0.0)) {\n            continue;\n        }",
        "the zero-intensity probe gate must directly exclude the candidate rather than only scale its radiance"
    );
}

#[test]
fn forward_environment_sphere_probe_weight_skips_rotation() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let weight = wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_probe_weight(");

    let position_delta = weight
        .find("let position_delta = world_position - probe.position_blend.xyz;")
        .expect("probe weighting must retain the world-space center delta");
    let sphere_branch = weight
        .find("if (probe.box_max.w >= 0.5) {")
        .expect("probe weighting must retain the spherical influence branch");
    let sphere_distance = weight
        .find("edge_distance = probe.box_max.x - length(position_delta);")
        .expect("spherical probe weighting must use the rotation-invariant center distance");
    let box_rotation = weight
        .find("let local_position = zr_environment_quat_rotate_inverse(")
        .expect("box probe weighting must retain local-space rotation");

    assert!(
        position_delta < sphere_branch
            && sphere_branch < sphere_distance
            && sphere_distance < box_rotation,
        "spherical probes must compute their influence before box-only inverse rotation"
    );
}

#[test]
fn forward_environment_skips_zero_weight_probe_samples() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_reflection_color_after_planar(",
    );

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
    let components =
        wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_components(");
    let prepared = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_pbr_prepared_inputs(",
    );

    let occlusion = prepared
        .find("let clamped_occlusion = clamp(occlusion, 0.0, 1.0);")
        .expect("environment PBR preparation should clamp occlusion");
    let early_out = prepared
        .find("if (clamped_occlusion <= 0.0) {")
        .expect("zero occlusion should skip environment sampling");
    let prepared_call = components
        .find("let prepared = zr_environment_pbr_prepared_inputs(")
        .expect("defensive PBR must prepare availability before normalizing inputs");
    let active = components
        .find("if (!prepared.is_active) {")
        .expect("defensive PBR must return when preparation rejects the provider set");
    let normalization = components
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("environment PBR components should normalize the normal after its early-out");
    assert!(
        occlusion < early_out && prepared_call < active && active < normalization,
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
        let components =
            wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_components(");
        let direction_components = if label == "generic" {
            wgsl_function_source(
                &assembly.wgsl_source,
                "fn zr_environment_pbr_components_with_prepared_inputs(",
            )
        } else {
            components
        };

        let normal = components
            .find("zr_environment_normalize_or_zero(normal_ws)")
            .unwrap_or_else(|| panic!("{label} PBR should normalize the normal"));
        let view = match label {
            "generic" => components
                .find("zr_environment_normalize_or_zero(view_dir_ws)")
                .expect("generic PBR should normalize an arbitrary view direction"),
            "environment-only" => components
                .find("let view_dir = view_dir_normalized;")
                .expect("environment-only PBR should reuse the caller-normalized view direction"),
            _ => unreachable!("unexpected Forward PBR test profile: {label}"),
        };
        let invalid_direction = direction_components
            .find("if (all(normal == vec3<f32>(0.0)) || all(view_dir == vec3<f32>(0.0))) {")
            .unwrap_or_else(|| panic!("{label} PBR should reject zero cube-sampling directions"));
        let reflection = direction_components
            .find("let reflection =")
            .unwrap_or_else(|| panic!("{label} PBR should resolve reflection after validation"));
        assert!(
            normal < view && invalid_direction < reflection,
            "{label} PBR must reject zero normal/view inputs before PMREM, SH, or BRDF work"
        );
        assert!(
            direction_components[invalid_direction..reflection]
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
    let clearcoat_environment =
        wgsl_function_source(&assembly.wgsl_source, "fn zr_pbr_advanced_environment(");
    let normalized_clearcoat_environment = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_pbr_advanced_environment_normalized(",
    );

    let coat_normal = clearcoat_environment
        .find("let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);")
        .expect("clearcoat environment should normalize its normal");
    let view_direction = clearcoat_environment
        .find("let normalized_view_dir = zr_normalize_or_zero(view_dir);")
        .expect("clearcoat environment should normalize its view direction");
    let normalized_call = clearcoat_environment
        .find("return zr_pbr_advanced_environment_normalized(")
        .expect("clearcoat environment should forward its normalized inputs");
    let invalid_direction = normalized_clearcoat_environment
        .find(
            "if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {",
        )
        .expect("clearcoat environment should reject zero reflection directions");
    let planar = normalized_clearcoat_environment
        .find("let planar = zr_environment_planar_reflection(")
        .expect(
            "clearcoat environment should resolve its planar reflection before provider fallback",
        );
    let availability = normalized_clearcoat_environment
        .find("let has_global_environment = zr_environment_is_enabled()")
        .expect("clearcoat environment should resolve global availability after a planar miss");
    let provider = normalized_clearcoat_environment
        .find("reflected = zr_environment_reflection_color_after_planar(")
        .expect("clearcoat environment should resolve reflection through normalized inputs");
    let zero_reflection = normalized_clearcoat_environment
        .find("if (all(reflected == vec3<f32>(0.0))) {")
        .expect("clearcoat environment should skip BRDF work for zero reflection radiance");
    let no_v = normalized_clearcoat_environment
        .find("let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);")
        .expect("clearcoat environment should evaluate NdotV only for nonzero reflections");
    let brdf_lut = normalized_clearcoat_environment
        .find("zr_environment_env_brdf_lut(")
        .expect("clearcoat environment should retain the EnvBRDF LUT for active reflections");

    assert!(
        coat_normal < view_direction && view_direction < normalized_call,
        "clearcoat must normalize its inputs before forwarding them to the advanced helper"
    );
    assert!(
        invalid_direction < planar && planar < availability && availability < provider,
        "clearcoat must reject zero normal/view inputs before planar, cubemap, or BRDF work"
    );
    assert!(
        normalized_clearcoat_environment[invalid_direction..provider]
            .contains("return vec3<f32>(0.0);"),
        "clearcoat must return before reflection sampling"
    );
    assert!(
        provider < zero_reflection && zero_reflection < no_v && no_v < brdf_lut,
        "zero clearcoat reflection must return before NdotV and EnvBRDF texture work"
    );
    assert!(
        normalized_clearcoat_environment[zero_reflection..no_v].contains("return vec3<f32>(0.0);"),
        "the clearcoat zero-reflection gate must return rather than only branch around the LUT"
    );
    assert!(
        !normalized_clearcoat_environment.contains("zr_environment_reflection_color("),
        "clearcoat normalized inputs must not enter the defensive reflection wrapper"
    );
}

#[test]
fn forward_clearcoat_zero_direction_preserves_base_layer_energy() {
    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::PBR_CLEARCOAT)),
    )
    .expect("clearcoat Forward template assembly");
    let clearcoat_energy = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_pbr_clearcoat_base_energy_scale(",
    );
    let normalized_clearcoat_energy = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_pbr_clearcoat_base_energy_scale_normalized(",
    );

    let coat_normal = clearcoat_energy
        .find("let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);")
        .expect("clearcoat base energy should normalize its normal");
    let view_direction = clearcoat_energy
        .find("let normalized_view_dir = zr_normalize_or_zero(view_dir);")
        .expect("clearcoat base energy should normalize its view direction");
    let normalized_call = clearcoat_energy
        .find("return zr_pbr_clearcoat_base_energy_scale_normalized(")
        .expect("clearcoat base energy should forward its normalized inputs");
    let invalid_direction = normalized_clearcoat_energy
        .find(
            "if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {",
        )
        .expect("clearcoat base energy should reject invalid directions");
    let no_v = normalized_clearcoat_energy
        .find("let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);")
        .expect("clearcoat base energy should use normalized directions");

    assert!(
        coat_normal < view_direction && view_direction < normalized_call,
        "clearcoat base energy must normalize inputs before forwarding them to the normalized helper"
    );
    assert!(
        invalid_direction < no_v,
        "clearcoat base energy must preserve the base layer before invalid-direction BRDF work"
    );
    assert!(
        normalized_clearcoat_energy[invalid_direction..no_v].contains("return vec3<f32>(1.0);"),
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
    let prepared = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_pbr_prepared_inputs(",
    );
    let reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_reflection_color_after_planar(",
    );

    assert!(
        !prepared.contains("if (!zr_environment_is_enabled() || !is_standard_pbr)"),
        "the global sky/source flag must not disable independent local reflection providers"
    );
    let material_gate = prepared
        .find("if (!is_standard_pbr) {")
        .expect("only the material model should unconditionally disable PBR indirect lighting");
    let global_availability = prepared
        .find(
            "let has_global_environment = zr_environment_is_enabled()\n        && environment_intensity > 0.0;",
        )
        .expect("PBR components should combine the global source flag and intensity");
    let provider_early_out = prepared
        .find(
            "if (!has_global_environment\n        && zr_env_probe_header.probe_count == 0u\n        && zr_env_planar.sample_params.w < 0.5)",
        )
        .expect("PBR components should return only when every environment provider is unavailable");
    assert!(
        material_gate < global_availability && global_availability < provider_early_out,
        "global availability must be resolved before the provider-aware early-out"
    );
    assert!(
        reflection.contains("has_global_environment: bool"),
        "the provider must consume the PBR entry's resolved global availability"
    );
    assert!(
        !reflection.contains("let has_global_environment ="),
        "the provider must not recompute global availability after PBR resolves it"
    );
}

#[test]
fn forward_environment_keeps_local_provider_intensity_independent_from_global_intensity() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let probe = wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_probe_color(");
    let indirect = wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_indirect(");

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
    let components =
        wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_components(");
    let prepared = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_pbr_prepared_inputs(",
    );

    let intensity = prepared
        .find("let environment_intensity = max(scene.environment_params.y, 0.0);")
        .expect("environment PBR components should clamp intensity");
    let global_availability = prepared
        .find(
            "let has_global_environment = zr_environment_is_enabled()\n        && environment_intensity > 0.0;",
        )
        .expect("environment PBR components should combine source availability and intensity");
    let early_out = prepared
        .find(
            "if (!has_global_environment\n        && zr_env_probe_header.probe_count == 0u\n        && zr_env_planar.sample_params.w < 0.5)",
        )
        .expect("zero intensity should skip global environment sampling only without local reflections");
    let normalization = components
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("environment PBR components should normalize the normal after its early-out");
    let prepared_call = components
        .find("let prepared = zr_environment_pbr_prepared_inputs(")
        .expect("defensive PBR must prepare providers before normalizing inputs");
    let active = components
        .find("if (!prepared.is_active) {")
        .expect("defensive PBR must reject unavailable providers before normalization");
    assert!(
        intensity < global_availability
            && global_availability < early_out
            && prepared_call < active
            && active < normalization,
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
    let reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_reflection_color_after_planar(",
    );
    let components = environment_pbr_composition_source(&assembly.wgsl_source);

    let global_availability = reflection
        .find("has_global_environment: bool")
        .expect("reflection mixing should receive resolved global availability");
    let first_sky_sample = reflection
        .find("zr_environment_sky_reflection_color(reflected, clamped_roughness)")
        .expect("reflection mixing should retain the global sky fallback");
    assert!(
        global_availability < first_sky_sample,
        "global availability must be resolved before any sky or PMREM sample"
    );
    assert!(reflection.contains(
        "if (zr_env_probe_header.probe_count == 0u && !has_global_environment) {\n        return vec3<f32>(0.0);"
    ));
    assert!(!reflection.contains("let has_global_environment ="));
    assert!(reflection.contains("if (sky_weight > 0.0 && has_global_environment) {"));
    assert!(components.contains(
        "if (diffuse_energy_scale > 0.0\n        && has_global_environment\n        && any(diffuse_color != vec3<f32>(0.0)))"
    ));
}

#[test]
fn forward_environment_defers_reflection_direction_until_a_provider_can_consume_it() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_reflection_color_after_planar(",
    );

    let availability = reflection
        .find("has_global_environment: bool")
        .expect("reflection composition should receive global availability first");
    let no_provider = reflection
        .find("if (zr_env_probe_header.probe_count == 0u && !has_global_environment) {")
        .expect("reflection composition should fast-path when every provider is unavailable");
    let zero_return = reflection[no_provider..]
        .find("return vec3<f32>(0.0);")
        .map(|offset| no_provider + offset)
        .expect("an unavailable global environment should return zero without probes");
    let reflected = reflection
        .find("let reflected = reflect(-view_dir, normal);")
        .expect("reflection composition should build the direction for active providers");

    assert!(
        availability < no_provider && no_provider < zero_return && zero_return < reflected,
        "an inactive planar sample with no global/probe provider must return before reflection ALU"
    );
}

#[test]
fn forward_environment_defers_defensive_global_availability_until_after_planar_hit_test() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let reflection =
        wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_reflection_color(");

    let planar = reflection
        .find("let planar = zr_environment_planar_reflection(")
        .expect("defensive reflection wrapper must test planar reflection first");
    let planar_return = reflection[planar..]
        .find("return planar.rgb;")
        .map(|offset| planar + offset)
        .expect("defensive reflection wrapper must return a valid planar reflection");
    let availability = reflection
        .find("let has_global_environment = zr_environment_is_enabled()")
        .expect(
            "defensive reflection wrapper must resolve global availability after a planar miss",
        );
    let no_provider = reflection
        .find("if (zr_env_probe_header.probe_count == 0u && !has_global_environment) {")
        .expect("defensive reflection wrapper must reject an unavailable provider set");
    let normal_normalization = reflection
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("defensive reflection wrapper must normalize its raw normal for active providers");
    let view_normalization = reflection
        .find("zr_environment_normalize_or_zero(view_dir_ws)")
        .expect("defensive reflection wrapper must normalize its raw view direction for active providers");

    assert!(
        planar < planar_return
            && planar_return < availability
            && availability < no_provider
            && no_provider < normal_normalization
            && normal_normalization < view_normalization,
        "a planar hit must return first, and an empty provider set must return before raw-input normalization"
    );
}

#[test]
fn forward_environment_skips_texture_work_for_zero_diffuse_and_reflection_contributions() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let provider_components = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_pbr_components_with_prepared_inputs(",
    );
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
    let provider_components = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_pbr_components_with_prepared_inputs(",
    );
    let components = environment_pbr_composition_source(&assembly.wgsl_source);
    let defensive_components =
        wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_components(");

    let normalized_normal = defensive_components
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("defensive PBR components should normalize the normal once");
    let diffuse = components
        .find("zr_environment_diffuse_color_normalized(normal)")
        .expect("PBR diffuse IBL should reuse the normalized normal");
    assert!(
        provider_components.contains("normal: vec3<f32>")
            && normalized_normal
                < defensive_components
                    .find("return zr_environment_pbr_components_with_prepared_inputs(")
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
    let specialized_source = wgsl_without_comments(&specialized.wgsl_source);

    assert!(specialized_source.contains("const ZR_FEATURE_ENVIRONMENT_ONLY_PBR: bool = true;"));
    assert!(specialized_source.contains("zr_environment_pbr_indirect("));
    for forbidden in [
        "zr_standard_pbr_gpu_light_lighting(",
        "zr_light_cookie_factor(",
        "zr_lightmap_baked_irradiance(",
        "zr_gpu_light_shadow_visibility(",
        "zr_volumetric_apply(",
    ] {
        assert!(
            !specialized_source.contains(forbidden),
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
fn environment_only_forward_shading_preserves_runtime_material_model_safety() {
    let shading = wgsl_function_source(
        include_str!("../../wgsl/zr_shading_environment_only_pbr.wgsl"),
        "fn shade_forward(",
    );
    let unlit_guard = shading
        .find("if (surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID)")
        .expect("environment-only Forward must preserve the unlit guard");
    let unlit_return = shading[unlit_guard..]
        .find("return surface.base_color.rgb + surface.emissive;")
        .map(|offset| unlit_guard + offset)
        .expect("environment-only Forward must return from its unlit guard");
    let ibl_call = shading
        .find("let environment_lights = zr_environment_pbr_indirect(")
        .expect("environment-only Forward must retain its IBL call");
    let ibl_call_end = shading[ibl_call..]
        .find("\n    );")
        .map(|offset| ibl_call + offset)
        .expect("environment-only Forward IBL call must close");

    assert!(
        unlit_guard < unlit_return && unlit_return < ibl_call,
        "environment-only Forward must return for raw feature-bit Unlit surfaces before IBL"
    );
    assert!(
        shading[ibl_call..ibl_call_end].contains(
            "surface.occlusion,\n        surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,",
        ),
        "environment-only Forward must pass the dynamic Standard-PBR gate to IBL"
    );
}
