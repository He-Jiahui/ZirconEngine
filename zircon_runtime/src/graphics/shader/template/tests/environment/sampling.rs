use super::*;

#[test]
fn generic_environment_brdf_approx_matches_unreal_f90_gate() {
    let generic_api = include_str!("../../../wgsl/zr_environment_generic_api.wgsl");
    let approximate = wgsl_function_source(generic_api, "fn zr_environment_env_brdf_approx(");

    assert!(approximate.contains("let f90 = clamp(50.0 * f0.g, 0.0, 1.0);"));
    assert!(approximate.contains("f0 * ab.x + vec3<f32>(f90) * ab.y"));
    assert!(
        !approximate.contains("f0 * ab.x + vec3<f32>(ab.y)"),
        "the generic approximation must not restore a fixed F90=1 term"
    );
}

#[test]
fn environment_brdf_helpers_preserve_unclamped_split_sum_energy() {
    let core = include_str!("../../../wgsl/zr_environment_core.wgsl");
    let generic_api = include_str!("../../../wgsl/zr_environment_generic_api.wgsl");
    let lut = wgsl_function_source(core, "fn zr_environment_env_brdf_lut(");
    let approximate = wgsl_function_source(generic_api, "fn zr_environment_env_brdf_approx(");

    for helper in [lut, approximate] {
        assert!(helper.contains("let f90 = clamp(50.0 * f0.g, 0.0, 1.0);"));
        assert!(helper.contains("return f0 * ab.x + vec3<f32>(f90) * ab.y;"));
        assert!(
            !helper.contains("return clamp("),
            "the split-sum result must not receive a post-integration energy clamp"
        );
    }
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
fn forward_environment_skips_global_sampling_when_intensity_is_not_positive_and_local_reflections_are_absent()
 {
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
        "if (diffuse_energy_scale > 0.0\n        && has_global_environment\n        && any(pbr_diffuse_color != vec3<f32>(0.0)))"
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
        .find("let perfect_reflection = zr_environment_perfect_specular_direction_normalized(")
        .expect(
            "reflection composition should build the reflection direction for active providers",
        );

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
            "if (any(diffuse_energy_scale > vec3<f32>(0.0))\n        && has_global_environment\n        && any(pbr_diffuse_color != vec3<f32>(0.0)))\n    {",
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
    let pbr_diffuse = "let pbr_diffuse_color = zr_pbr_base_color(diffuse_color);";
    let pbr_base = "let pbr_base_color = zr_pbr_base_color(base_color);";
    let f0 = "let f0 = zr_pbr_material_f0(dielectric_f0, pbr_base_color, clamped_metallic);";
    let no_v = "let no_v = clamp(dot(normal, view_dir), 0.0, 1.0);";
    let diffuse_energy = "let diffuse_energy_scale = vec3<f32>(\n        zr_surface_metallic_diffuse_energy_scale(clamped_metallic),\n    );";
    let no_v_offset = components
        .find(no_v)
        .expect("PBR components should calculate NdotV before the split-sum specular lookup");
    let pbr_diffuse_offset = components
        .find(pbr_diffuse)
        .expect("PBR components should clamp diffuse reflectance once at the shared boundary");
    let pbr_base_offset = components
        .find(pbr_base)
        .expect("PBR components should clamp base reflectance once at the shared boundary");
    let f0_offset = components
        .find(f0)
        .expect("PBR components should derive the material F0 once");
    let diffuse_energy_offset = components
        .find(diffuse_energy)
        .expect("PBR components should derive metallic diffuse energy once");
    let diffuse_gate = components
        .find("if (any(diffuse_energy_scale > vec3<f32>(0.0))")
        .expect("PBR components should guard diffuse IBL with its spectral energy scale");
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
        no_v_offset < pbr_diffuse_offset
            && pbr_diffuse_offset < pbr_base_offset
            && pbr_base_offset < f0_offset
            && f0_offset < diffuse_energy_offset
            && diffuse_energy_offset < diffuse_gate,
        "material inputs and the metallic diffuse-energy gate must precede diffuse IBL sampling"
    );
    assert!(components.contains("zr_surface_metallic_diffuse_energy_scale(clamped_metallic),"));
    assert!(!components.contains("zr_pbr_diffuse_energy_scale("));
    assert!(reflection_branch.contains(brdf_sample));
    assert!(!reflection_branch.contains("let f0 ="));
    assert!(!reflection_branch.contains("let no_v ="));
    assert_eq!(components.matches(f0).count(), 1);
    assert_eq!(components.matches(pbr_diffuse).count(), 1);
    assert_eq!(components.matches(pbr_base).count(), 1);
    assert_eq!(components.matches(no_v).count(), 1);
    assert_eq!(components.matches(diffuse_energy).count(), 1);
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
        include_str!("../../../wgsl/zr_shading_environment_only_pbr.wgsl"),
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
            "surface.dielectric_f0,\n        surface.occlusion,\n        surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,",
        ),
        "environment-only Forward must pass material F0 and the dynamic Standard-PBR gate to IBL"
    );
}
