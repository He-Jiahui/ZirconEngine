use super::*;

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
    fn environment_hash(
        assembly: &super::super::super::assemble::MaterialShaderTemplateAssembly,
    ) -> &str {
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

    let reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_reflection_color_after_planar(",
    );
    for required in [
        "let perfect_reflection = zr_environment_perfect_specular_direction_normalized(",
        "let sky_has_pmrem = zr_environment_is_source_cubemap()",
        "zr_env_probe_header.probe_count > 0u || sky_has_pmrem",
        "sky_direction = dominant_direction;",
    ] {
        assert!(
            reflection.contains(required),
            "reflection routing must retain provider-specific direction contract `{required}`"
        );
    }

    let radiance = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_radiance_color_normalized(",
    );
    assert!(
        radiance.contains("zr_environment_sky_reflection_color(sky_direction, clamped_roughness)")
    );
    assert!(
        radiance.contains(
            "world_position,\n            probe_direction,\n            clamped_roughness,"
        )
    );
}

#[test]
fn environment_capture_rejects_unfiltered_procedural_specular_without_pmrem() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let sky_reflection = wgsl_function_source(
        &assembly.wgsl_source,
        "fn zr_environment_sky_reflection_color(",
    );

    let pmrem = sky_reflection
        .find("if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl())")
        .expect("ready PMREM providers must retain their roughness path");
    let capture = sky_reflection
        .find("if (scene.sky_sun_params.w > 0.5)")
        .expect("capture must identify the full-roughness surface policy");
    let fail_closed = sky_reflection[capture..]
        .find("return vec3<f32>(0.0);")
        .expect("capture without a PMREM must reject unfiltered specular");
    let viewport_fallback = sky_reflection
        .find("return zr_environment_procedural_sky_color_normalized(reflected);")
        .expect("ordinary viewport fallback must retain the reflected sky direction");

    assert!(pmrem < capture);
    assert!(capture + fail_closed < viewport_fallback);
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
fn forward_environment_consumes_probe_and_camera_layer_masks_before_weighting() {
    let assembly = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward template assembly");
    let selection = wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_select_probes(");

    let intensity_gate = selection
        .find("if (!(probe.misc.x > 0.0))")
        .expect("probe selection must retain the intensity gate");
    let layer_gate = selection
        .find("probe_layer_mask & zr_env_probe_header.camera_layer_mask")
        .expect("probe selection must consume the camera layer mask");
    let spatial_weight = selection
        .find("let weight = zr_environment_probe_weight(probe, world_position);")
        .expect("probe selection must retain spatial weighting");

    assert!(intensity_gate < layer_gate && layer_gate < spatial_weight);
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
    let environment_only_features = ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR);
    let environment_only_surface =
        standard_material_surface_source_for_features(environment_only_features, 0.5);
    let environment_only_request = MaterialShaderTemplateRequest::new(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
        environment_only_surface.source,
        environment_only_surface.entry_point,
    )
    .with_features(environment_only_surface.features);

    for (label, request) in [
        (
            "generic",
            material_template_request(static_mesh_descriptor(), ShaderPassType::Forward),
        ),
        ("environment-only", environment_only_request),
    ] {
        let assembly = assemble_material_shader_template(request)
            .unwrap_or_else(|error| panic!("{label} Forward template assembly: {error:?}"));
        let components =
            wgsl_function_source(&assembly.wgsl_source, "fn zr_environment_pbr_components(");
        let prepared_direction_components = (label == "generic").then(|| {
            wgsl_function_source(
                &assembly.wgsl_source,
                "fn zr_environment_pbr_components_with_prepared_inputs(",
            )
        });
        let direction_components = prepared_direction_components
            .as_deref()
            .unwrap_or(components.as_str());

        let normal = match label {
            "generic" => components
                .find("zr_environment_normalize_or_zero(normal_ws)")
                .expect("generic PBR should normalize an arbitrary normal"),
            "environment-only" => components
                .find("let normal = normal_normalized;")
                .expect("environment-only PBR should reuse the caller-normalized normal"),
            _ => unreachable!("unexpected Forward PBR test profile: {label}"),
        };
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
