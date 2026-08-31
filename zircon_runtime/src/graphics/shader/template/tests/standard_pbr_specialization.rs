use super::*;

fn include_content_hash<'a>(
    assembly: &'a super::super::assemble::MaterialShaderTemplateAssembly,
    token: &str,
) -> &'a str {
    assembly
        .include_tokens
        .iter()
        .zip(&assembly.include_content_hashes)
        .find_map(|(include_token, hash)| (include_token == token).then_some(hash.as_str()))
        .unwrap_or_else(|| panic!("Forward template should retain canonical `{token}` include"))
}

fn standard_pbr_forward_request(features: ShaderFeatureBits) -> MaterialShaderTemplateRequest {
    let surface_source = standard_material_surface_source_for_features(features, 0.5);
    MaterialShaderTemplateRequest::new(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
        surface_source.source,
        surface_source.entry_point,
    )
    .with_features(surface_source.features)
}

#[test]
fn forward_clearcoat_attenuates_emission_with_the_existing_base_layer_weight() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");
    let clearcoat = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::new(ShaderFeatureBits::PBR_CLEARCOAT),
    ))
    .expect("clearcoat Standard-PBR Forward template assembly");

    assert!(
        basic
            .wgsl_source
            .contains("+ environment_lights\n        + surface.emissive;")
    );
    assert!(!basic.wgsl_source.contains("clearcoat_base_energy"));

    for required in [
        "clearcoat_base_energy = zr_pbr_clearcoat_base_energy_scale_normalized(",
        "+ surface.emissive * clearcoat_base_energy;",
    ] {
        assert!(
            clearcoat.wgsl_source.contains(required),
            "clearcoat Forward PBR must reuse its KHR base-layer weight for `{required}`"
        );
    }
    assert!(
        clearcoat
            .wgsl_source
            .contains("return surface.base_color.rgb + surface.emissive;")
    );
    assert!(
        !clearcoat
            .wgsl_source
            .contains("+ transmitted_scene\n        + surface.emissive;")
    );

    for required in [
        "let coat_fresnel = zr_pbr_fresnel_schlick(no_v, vec3<f32>(0.04));",
        "return vec3<f32>(1.0) - coat_fresnel * clamp(surface.clearcoat, 0.0, 1.0);",
    ] {
        assert!(
            clearcoat.wgsl_source.contains(required),
            "clearcoat Forward PBR must retain the KHR layer formula `{required}`"
        );
    }

    let base_layer_weight = |coat: f32, no_v: f32| {
        let fresnel = 0.04 + 0.96 * (1.0 - no_v).powi(5);
        1.0 - coat * fresnel
    };
    for (coat, no_v, expected) in [(1.0, 1.0, 0.96), (1.0, 0.0, 0.0), (0.5, 1.0, 0.98)] {
        assert!((base_layer_weight(coat, no_v) - expected).abs() <= f32::EPSILON);
    }
}

#[test]
fn forward_pbr_clamps_metallic_f0_base_color_to_the_physical_reflectance_domain() {
    for (label, features) in [
        ("basic", ShaderFeatureBits::default()),
        (
            "advanced",
            ShaderFeatureBits::new(ShaderFeatureBits::PBR_CLEARCOAT),
        ),
    ] {
        let assembly = assemble_material_shader_template(standard_pbr_forward_request(features))
            .unwrap_or_else(|error| panic!("{label} Standard-PBR Forward assembly: {error:?}"));

        assert!(
            assembly
                .wgsl_source
                .contains("clamp(base_color, vec3<f32>(0.0), vec3<f32>(1.0))"),
            "{label} Forward PBR must prevent tint or vertex-color amplification from creating metallic F0 above one"
        );
        assert!(
            !assembly
                .wgsl_source
                .contains("max(base_color, vec3<f32>(0.0))"),
            "{label} Forward PBR must not retain the lower-bound-only metallic F0 input"
        );
    }
}

#[test]
fn forward_pbr_clamps_diffuse_reflectance_without_changing_legacy_base_color_semantics() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");
    let transmission = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::new(ShaderFeatureBits::PBR_TRANSMISSION),
    ))
    .expect("transmission Standard-PBR Forward template assembly");

    for required in [
        "fn zr_pbr_base_color(base_color: vec3<f32>) -> vec3<f32>",
        "return clamp(base_color, vec3<f32>(0.0), vec3<f32>(1.0));",
        "let pbr_diffuse_color = zr_pbr_base_color(diffuse_color);",
        "let pbr_base_color = zr_pbr_base_color(base_color);",
    ] {
        assert!(
            basic.wgsl_source.contains(required),
            "Forward PBR must retain the shared physical base-color contract `{required}`"
        );
    }

    let diffuse_owner = basic
        .wgsl_source
        .split("fn zr_standard_pbr_diffuse_color(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_ambient_diffuse_energy_scale(")
                .next()
        })
        .expect("Forward PBR must retain a diffuse-color owner");
    for required in [
        "surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "return zr_pbr_base_color(surface.base_color.rgb);",
        "return surface.base_color.rgb;",
    ] {
        assert!(
            diffuse_owner.contains(required),
            "Forward diffuse routing must retain `{required}`"
        );
    }
    assert!(
        basic
            .wgsl_source
            .contains("return surface.base_color.rgb + surface.emissive;"),
        "Unlit must keep its authored base-color semantics"
    );

    let transmission_lobe = transmission
        .wgsl_source
        .split("fn zr_standard_pbr_shade_standard_light_vector_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_standard_pbr_shade_blinn_phong_light_vector_normalized(")
                .next()
        })
        .expect("transmission PBR must retain the Standard-PBR direct-light owner");
    assert!(transmission_lobe.contains("transmitted_diffuse = zr_transmission_btdf("));
    assert!(transmission_lobe.contains("light_vector,\n            diffuse_color,"));
    assert!(transmission_lobe.contains(") * radiance * volume_attenuation;"));
}

#[test]
fn forward_pbr_uses_source_independent_metallic_diffuse_and_ggx_specular() {
    for (label, features) in [
        ("basic", ShaderFeatureBits::default()),
        (
            "advanced",
            ShaderFeatureBits::new(ShaderFeatureBits::PBR_ANISOTROPY),
        ),
    ] {
        let assembly = assemble_material_shader_template(standard_pbr_forward_request(features))
            .unwrap_or_else(|error| panic!("{label} Standard-PBR Forward assembly: {error:?}"));

        let diffuse_energy_owner_name = "fn zr_surface_metallic_diffuse_energy_scale(";
        assert_eq!(
            assembly
                .wgsl_source
                .matches(diffuse_energy_owner_name)
                .count(),
            1,
            "{label} Forward PBR must assemble exactly one metallic diffuse-energy owner"
        );
        let diffuse_energy_owner = assembly
            .wgsl_source
            .split(diffuse_energy_owner_name)
            .nth(1)
            .and_then(|source| source.split("fn zr_pbr_fresnel_schlick(").next())
            .expect("Forward PBR must assemble metallic diffuse energy in the common PBR module");
        for required in ["metallic: f32", "return 1.0 - clamp(metallic, 0.0, 1.0);"] {
            assert!(
                diffuse_energy_owner.contains(required),
                "{label} diffuse energy owner must retain `{required}`"
            );
        }

        for required in [
            "fn zr_pbr_isotropic_ggx(",
            "let specular = zr_pbr_isotropic_ggx(",
        ] {
            assert!(
                assembly.wgsl_source.contains(required),
                "{label} Forward PBR must retain the shared GGX specular contract `{required}`"
            );
        }
        if label == "advanced" {
            assert!(assembly.wgsl_source.contains("specular = zr_aniso_ggx("));
            assert!(
                assembly
                    .wgsl_source
                    .contains("let reflected_diffuse =\n        direct_diffuse_brdf")
            );
        } else {
            assert!(
                assembly
                    .wgsl_source
                    .contains("return (direct_diffuse_brdf + specular) * radiance * no_l;")
            );
        }
        for rejected in [
            "struct ZrPbrSpecularComponents",
            "fn zr_pbr_isotropic_ggx_components(",
            "fn zr_aniso_ggx_components(",
            "specular_components.fresnel",
        ] {
            assert!(
                !assembly.wgsl_source.contains(rejected),
                "{label} Forward PBR must reject source-dependent diffuse contract `{rejected}`"
            );
        }

        let ambient_owner = assembly
            .wgsl_source
            .split("fn zr_standard_pbr_ambient_diffuse_energy_scale(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn zr_standard_pbr_shade_standard_light_vector_normalized(")
                    .next()
            })
            .expect("Forward PBR must retain the legacy-model ambient routing owner");
        for required in [
            "surface: ZrSurfaceOutput",
            "return vec3<f32>(",
            "zr_surface_metallic_diffuse_energy_scale(surface.metallic),",
        ] {
            assert!(
                ambient_owner.contains(required),
                "{label} ambient diffuse owner must retain `{required}`"
            );
        }

        let environment_owner = assembly
            .wgsl_source
            .split("fn zr_environment_pbr_components_from_reflection(")
            .nth(1)
            .expect("Forward PBR must retain the environment PBR component owner");
        assert!(
            environment_owner
                .contains("zr_surface_metallic_diffuse_energy_scale(clamped_metallic),")
        );
    }
}

#[test]
fn forward_pbr_transmission_replaces_only_the_base_diffuse_layer() {
    let layered =
        assemble_material_shader_template(standard_pbr_forward_request(ShaderFeatureBits::new(
            ShaderFeatureBits::PBR_TRANSMISSION | ShaderFeatureBits::PBR_CLEARCOAT,
        )))
        .expect("layered transmission Standard-PBR Forward template assembly");

    for required in [
        "let specular_transmission = select(",
        "clamp(surface.specular_transmission, 0.0, 1.0),",
        "let diffuse_transmission = select(",
        "let reflected_diffuse_weight = (1.0 - specular_transmission)",
        "* (1.0 - diffuse_transmission);",
        "var diffuse_transmission_attenuation = vec3<f32>(1.0);",
        "if (diffuse_transmission > 0.0) {",
        "diffuse_transmission_attenuation = zr_pbr_volume_attenuation(",
        "var transmitted_indirect_diffuse = vec3<f32>(0.0);",
        "if (zr_environment_is_enabled() && scene.environment_params.y > 0.0) {",
        "zr_environment_diffuse_color_normalized(-world_normal)",
        "* diffuse_transmission_attenuation;",
        "let reflected_diffuse =\n        direct_diffuse_brdf",
        "* reflected_diffuse_weight",
        "* direct_base_energy",
        "surface.diffuse_transmission > 0.0",
        "if (ZR_FEATURE_PBR_TRANSMISSION && diffuse_transmission > 0.0)",
        "zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized(",
        "let base_diffuse_lighting =",
        "environment_components.diffuse",
        "+ transmitted_indirect_diffuse",
        "let retained_reflection_lighting = direct_lights",
        "+ environment_components.specular * clearcoat_base_energy",
        "let transmission_frame = zr_pbr_transmission_frame_normalized(",
        "let transmission_energy = zr_pbr_transmission_energy_scale(",
        "transmission_frame.fresnel_cosine,",
        "let transmission_tint = zr_pbr_base_color(surface.base_color.rgb)",
        "* transmission_energy;",
        "let specular_transmission_attenuation = zr_pbr_volume_attenuation(",
        "specular_transmission_attenuation,",
        "transmitted_scene * clearcoat_base_energy",
    ] {
        assert!(
            layered.wgsl_source.contains(required),
            "layered transmission must retain `{required}`"
        );
    }

    for rejected in [
        "opaque_lighting * (1.0 - specular_transmission)",
        "let refraction_scale = max(surface.ior - 1.0, 0.0)",
        "normal_ws).xy * refraction_scale",
        "let transmission_environment = zr_environment_transmission_radiance_normalized(",
        "scene_color_sample.a > 0.0",
        "* zr_surface_metallic_diffuse_energy_scale(surface.metallic);",
        "zr_pbr_screen_space_transmission(\n            surface,\n            ctx.position_ws,\n            environment_lights,",
    ] {
        assert!(
            !layered.wgsl_source.contains(rejected),
            "layered transmission must reject aggregate-lobe composition `{rejected}`"
        );
    }
}

#[test]
fn forward_pbr_transmission_uses_one_thin_or_scaled_volume_frame() {
    let layered = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::new(ShaderFeatureBits::PBR_TRANSMISSION),
    ))
    .expect("transmission Standard-PBR Forward template assembly");

    for required in [
        "struct ZrPbrTransmissionFrame {",
        "transmission_distance: f32,",
        "fn zr_pbr_transmission_frame_normalized(",
        "instance_index: u32,",
        "if (thickness <= 0.0) {",
        "let world_from_local = zr_world_from_local(instance_index);",
        "let refracted_direction = refract(-view_dir, normal, inverse_ior);",
        "let model_scale = vec3<f32>(",
        "length(world_from_local[0].xyz),",
        "length(world_from_local[1].xyz),",
        "length(world_from_local[2].xyz),",
        "let transmission_ray = zr_pbr_normalize_or_zero(refracted_direction)",
        "let exit_position = world_position + transmission_ray;",
        "let environment_direction = -zr_pbr_normalize_or_zero(",
        "view_dir - transmission_ray,",
        "let transmission_distance = length(transmission_ray);",
        "let fresnel_cosine = clamp(dot(normal, view_dir), 0.0, 1.0);",
        "struct ZrPbrViewportProjection {",
        "let refracted_projection = zr_pbr_viewport_projection(",
        "if (refracted_projection.valid) {",
        "transmission_frame.exit_position,",
        "transmission_frame.environment_direction,",
        "fn zr_pbr_volume_attenuation(",
        "const ZR_PBR_NO_ATTENUATION_DISTANCE: f32 = 1.0e30;",
        "if (transmission_distance <= 0.0",
        "|| surface.attenuation_distance >= ZR_PBR_NO_ATTENUATION_DISTANCE)",
        "let attenuation_power = transmission_distance / attenuation_distance;",
        "return vec3<f32>(1.0);",
        "clamp(surface.attenuation_color, vec3<f32>(0.0), vec3<f32>(1.0)),",
        "ctx.instance_index,",
    ] {
        assert!(
            layered.wgsl_source.contains(required),
            "transmission frame must retain `{required}`"
        );
    }

    for rejected in [
        "* max(surface.thickness, 0.0)\n        * 0.02",
        "base_uv + zr_normalize_or_zero(surface.normal_ws).xy",
        "let refracted = refract(-view_dir, normal, 1.0 / max(ior, 1.0));",
        "world_position + refracted_direction * thickness",
        "max(surface.thickness, 0.0) / attenuation_distance",
        "max(surface.attenuation_color, vec3<f32>(ZR_PBR_EXTRAS_EPSILON))",
        "world_from_local: mat4x4<f32>,",
        "zr_world_from_local(ctx.instance_index)",
        "let safe_w =",
        "clamp(\n        vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5)",
    ] {
        assert!(
            !layered.wgsl_source.contains(rejected),
            "transmission frame must reject `{rejected}`"
        );
    }
}

#[test]
fn pbr_volume_transmission_distance_scales_from_mesh_to_world_space() {
    fn transmission_distance(direction: [f32; 3], thickness: f32, model_scale: [f32; 3]) -> f32 {
        let ray = [
            direction[0] * thickness * model_scale[0],
            direction[1] * thickness * model_scale[1],
            direction[2] * thickness * model_scale[2],
        ];
        (ray[0] * ray[0] + ray[1] * ray[1] + ray[2] * ray[2]).sqrt()
    }

    fn beer_lambert(color: f32, distance: f32, attenuation_distance: f32) -> f32 {
        const NO_ATTENUATION_DISTANCE: f32 = 1.0e30;
        if distance <= 0.0 || attenuation_distance >= NO_ATTENUATION_DISTANCE {
            1.0
        } else {
            color.clamp(0.0, 1.0).powf(distance / attenuation_distance)
        }
    }

    let axis = [1.0, 0.0, 0.0];
    assert_eq!(transmission_distance(axis, 0.0, [4.0, 2.0, 1.0]), 0.0);
    assert_eq!(transmission_distance(axis, 1.0, [1.0; 3]), 1.0);
    assert_eq!(transmission_distance(axis, 1.0, [2.0; 3]), 2.0);

    let diagonal = [std::f32::consts::FRAC_1_SQRT_2; 2];
    let non_uniform_distance =
        transmission_distance([diagonal[0], diagonal[1], 0.0], 1.0, [2.0, 1.0, 1.0]);
    assert!((non_uniform_distance - 2.5_f32.sqrt()).abs() < 0.000001);

    let attenuation = beer_lambert(0.25, transmission_distance(axis, 1.0, [2.0; 3]), 2.0);
    assert!((attenuation - 0.25).abs() < 0.000001);
    assert_eq!(beer_lambert(0.0, 0.0, 1.0), 1.0);
    assert_eq!(beer_lambert(0.0, 1.0, 1.0), 0.0);
    assert_eq!(beer_lambert(0.0, 1.0, 1.0e30), 1.0);
    assert_eq!(beer_lambert(0.0, 1.0, f32::MAX), 1.0);
}

#[test]
fn anisotropic_forward_owns_an_explicit_environment_specular_normal() {
    let anisotropic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::new(ShaderFeatureBits::PBR_ANISOTROPY),
    ))
    .expect("anisotropic Standard-PBR Forward template assembly");

    for required in [
        "fn zr_pbr_anisotropic_environment_normal_normalized(",
        "let anisotropic_bitangent = zr_pbr_normalize_or_zero(cross(",
        "let anisotropic_normal = zr_pbr_normalize_or_zero(cross(",
        "let bend_factor = 1.0 - strength * (1.0 - roughness);",
        "let strength = clamp(anisotropy_strength, 0.0, 1.0);",
        "let bend_factor_pow4 = bend_factor_sq * bend_factor_sq;",
        "fn zr_environment_reflection_color_from_direction_normalized(",
        "fn zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized(",
        "let reflection = zr_environment_reflection_color_normalized(",
        "var environment_specular_normal = world_normal;",
        "surface.anisotropy_strength > 0.0",
        "environment_specular_normal,",
    ] {
        assert!(
            anisotropic.wgsl_source.contains(required),
            "anisotropic IBL must retain `{required}`"
        );
    }

    assert!(
        anisotropic
            .wgsl_source
            .contains("zr_environment_pbr_components_from_reflection(\n        normal_normalized,")
    );

    let environment_owner = anisotropic
        .wgsl_source
        .split(
            "fn zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized(",
        )
        .nth(1)
        .expect("anisotropic IBL must retain its dedicated environment-components owner");
    assert!(
        environment_owner.contains("let reflection = zr_environment_reflection_color_normalized(")
    );
    assert!(!environment_owner.contains(
        "let specular_direction = zr_environment_dominant_specular_direction_normalized("
    ));
}

#[test]
fn anisotropic_environment_direction_preserves_isotropic_and_roughness_endpoints() {
    fn assert_vec3_close(actual: [f32; 3], expected: [f32; 3]) {
        for channel in 0..3 {
            assert!(
                (actual[channel] - expected[channel]).abs() <= 0.000001,
                "actual={actual:?}, expected={expected:?}"
            );
        }
    }

    fn normalize(value: [f32; 3]) -> [f32; 3] {
        let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
        [value[0] / length, value[1] / length, value[2] / length]
    }

    fn mix(lhs: [f32; 3], rhs: [f32; 3], weight: f32) -> [f32; 3] {
        [
            lhs[0] * (1.0 - weight) + rhs[0] * weight,
            lhs[1] * (1.0 - weight) + rhs[1] * weight,
            lhs[2] * (1.0 - weight) + rhs[2] * weight,
        ]
    }

    fn bent_normal(
        normal: [f32; 3],
        anisotropic_normal: [f32; 3],
        roughness: f32,
        strength: f32,
    ) -> [f32; 3] {
        let bend_factor = 1.0 - strength * (1.0 - roughness);
        normalize(mix(anisotropic_normal, normal, bend_factor.powi(4)))
    }

    fn off_specular_direction(
        reflected: [f32; 3],
        bent_normal: [f32; 3],
        roughness: f32,
    ) -> [f32; 3] {
        normalize(mix(reflected, bent_normal, roughness * roughness))
    }

    let normal = [0.0, 0.0, 1.0];
    let anisotropic_normal = normalize([1.0, 0.0, 1.0]);
    assert_vec3_close(bent_normal(normal, anisotropic_normal, 0.0, 0.0), normal);
    assert_vec3_close(bent_normal(normal, anisotropic_normal, 1.0, 1.0), normal);
    assert_vec3_close(
        bent_normal(normal, anisotropic_normal, 0.0, 1.0),
        anisotropic_normal,
    );

    let reflected = normalize([1.0, 0.0, 1.0]);
    assert_vec3_close(off_specular_direction(reflected, normal, 0.0), reflected);
    assert_vec3_close(off_specular_direction(reflected, normal, 1.0), normal);
}

#[test]
fn pbr_diffuse_energy_is_source_independent_and_metallic_owned() {
    let diffuse_energy =
        |_cos_theta: f32, _dielectric_f0: f32, metallic: f32| 1.0 - metallic.clamp(0.0, 1.0);

    assert_eq!(diffuse_energy(1.0, 0.04, 0.0), 1.0);
    assert_eq!(diffuse_energy(1.0, 0.04, 0.5), 0.5);
    assert_eq!(diffuse_energy(1.0, 0.04, 1.0), 0.0);

    let grazing_cosines = [0.0_f32, 0.25, 0.5, 1.0];
    let dielectric_f0_values = [0.0_f32, 0.04, 0.16, 1.0];
    for cosine in grazing_cosines {
        for dielectric_f0 in dielectric_f0_values {
            assert_eq!(diffuse_energy(cosine, dielectric_f0, 0.25), 0.75);
        }
    }
}

#[test]
fn forward_pbr_keeps_unreal_joint_smith_approximation_in_the_base_closure() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");
    let anisotropic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::new(ShaderFeatureBits::PBR_ANISOTROPY),
    ))
    .expect("anisotropic Standard-PBR Forward template assembly");

    for (label, assembly) in [("basic", &basic), ("anisotropic", &anisotropic)] {
        for required in [
            "fn zr_pbr_normalize_or_zero(",
            "let half_dir = zr_pbr_normalize_or_zero(view_dir + light_dir);",
            "fn zr_pbr_smith_joint_visibility_approx(",
            "let visibility_v = no_l * (no_v * (1.0 - alpha) + alpha);",
            "let visibility_l = no_v * (no_l * (1.0 - alpha) + alpha);",
            "let visibility = zr_pbr_smith_joint_visibility_approx(no_v, no_l, alpha);",
        ] {
            assert!(
                assembly.wgsl_source.contains(required),
                "{label} Forward PBR must retain the Unreal-compatible isotropic visibility `{required}`"
            );
        }
    }

    for exact_anisotropic_source in [
        "fn zr_pbr_smith_joint_visibility_anisotropic(",
        "if (strength <= ZR_PBR_EXTRAS_EPSILON) {",
        "return zr_pbr_isotropic_ggx(",
        "let visibility_v = no_l * length(vec3<f32>(",
        "alpha_t * to_v,",
        "alpha_b * bo_v,",
        "let visibility_l = no_v * length(vec3<f32>(",
        "alpha_t * to_l,",
        "alpha_b * bo_l,",
        "let base_alpha = max(perceptual_roughness * perceptual_roughness, 0.001);",
        "let alpha_t = mix(base_alpha, 1.0, strength * strength);",
        "let alpha_b = base_alpha;",
        "let distribution_vector = vec3<f32>(",
        "if (distribution_length_squared > 0.0) {",
        "let visibility = zr_pbr_smith_joint_visibility_anisotropic(",
    ] {
        assert!(
            !basic.wgsl_source.contains(exact_anisotropic_source),
            "basic Forward PBR must exclude advanced visibility source `{exact_anisotropic_source}`"
        );
        assert!(
            anisotropic.wgsl_source.contains(exact_anisotropic_source),
            "anisotropic Forward PBR must retain directional Smith visibility `{exact_anisotropic_source}`"
        );
    }
    for rejected_scalar_approximation in [
        "fn zr_pbr_smith_joint_visibility(",
        "sqrt(alpha_t * alpha_b)",
        "base_alpha * (1.0 + strength)",
        "base_alpha * (1.0 - strength)",
        "max(perceptual_roughness * perceptual_roughness, 0.002)",
        "ZR_PBR_EXTRAS_PI * alpha_t * alpha_b * denominator * denominator",
    ] {
        assert!(
            !anisotropic
                .wgsl_source
                .contains(rejected_scalar_approximation),
            "anisotropic Forward PBR must reject geometric-mean visibility `{rejected_scalar_approximation}`"
        );
    }
}

#[test]
fn anisotropic_ggx_uses_khronos_axis_roughness_and_isotropic_endpoint() {
    fn axes(perceptual_roughness: f32, strength: f32) -> (f32, f32) {
        let material_alpha = (perceptual_roughness * perceptual_roughness).max(0.001);
        let clamped_strength = strength.clamp(0.0, 1.0);
        let alpha_t = material_alpha * (1.0 - clamped_strength * clamped_strength)
            + clamped_strength * clamped_strength;
        (alpha_t, material_alpha)
    }

    fn distribution(alpha_t: f32, alpha_b: f32, to_h: f32, bo_h: f32, no_h: f32) -> f32 {
        let alpha_product = alpha_t * alpha_b;
        let distribution_vector = [alpha_b * to_h, alpha_t * bo_h, alpha_product * no_h];
        let denominator = distribution_vector
            .iter()
            .map(|component| component * component)
            .sum::<f32>();
        let scale = alpha_product / denominator;
        alpha_product * scale * scale / std::f32::consts::PI
    }

    let (isotropic_t, isotropic_b) = axes(0.5, 0.0);
    assert_eq!((isotropic_t, isotropic_b), (0.25, 0.25));
    let (full_t, full_b) = axes(0.5, 1.0);
    assert_eq!((full_t, full_b), (1.0, 0.25));
    let (near_zero_t, near_zero_b) = axes(0.0, 0.0001);
    assert!((near_zero_t - 0.00100001).abs() < 0.0000001);
    assert_eq!(near_zero_b, 0.001);

    let isotropic_peak = distribution(isotropic_t, isotropic_b, 0.0, 0.0, 1.0);
    assert!((isotropic_peak - 1.0 / (std::f32::consts::PI * 0.25 * 0.25)).abs() < 0.00001);
    let full_peak = distribution(full_t, full_b, 0.0, 0.0, 1.0);
    assert!((full_peak - 1.0 / (std::f32::consts::PI * full_t * full_b)).abs() < 0.00001);
    assert!(isotropic_peak > full_peak);
}

#[test]
fn forward_pbr_preserves_the_valid_low_roughness_ggx_distribution_peak() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");

    for required in [
        "let no_v = clamp(dot(normal, view_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);",
        "let no_l = clamp(dot(normal, light_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);",
        "let no_h = clamp(dot(normal, half_dir), 0.0, 1.0);",
        "let vo_h = clamp(dot(view_dir, half_dir), 0.0, 1.0);",
        "let distribution = alpha_squared / (ZR_PBR_EXTRAS_PI * denominator * denominator);",
    ] {
        assert!(
            basic.wgsl_source.contains(required),
            "base Forward PBR must retain the bounded Unreal-compatible GGX term `{required}`"
        );
    }
    assert!(
        !basic
            .wgsl_source
            .contains("let distribution = alpha_squared / max("),
        "the positive min-alpha contract must not be overridden by a 1e-6 distribution-denominator floor"
    );
}

#[test]
fn forward_base_pbr_specializes_advanced_source_out_of_its_compilation_closure() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");
    let advanced_variants = [
        ("clearcoat", ShaderFeatureBits::PBR_CLEARCOAT),
        ("anisotropy", ShaderFeatureBits::PBR_ANISOTROPY),
        ("transmission", ShaderFeatureBits::PBR_TRANSMISSION),
    ]
    .map(|(label, feature)| {
        let assembly = assemble_material_shader_template(standard_pbr_forward_request(
            ShaderFeatureBits::new(feature),
        ))
        .unwrap_or_else(|error| {
            panic!("{label} Standard-PBR Forward template assembly: {error:?}")
        });
        (label, assembly)
    });

    for assembly in [&basic]
        .into_iter()
        .chain(advanced_variants.iter().map(|(_, assembly)| assembly))
    {
        assert_include_token!(assembly, "zr_pbr_extras.wgsl");
        assert_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
        for required in [
            "fn zr_pbr_fresnel_schlick(",
            "fn zr_pbr_smith_joint_visibility_approx(",
            "fn zr_pbr_isotropic_ggx(",
            "fn zr_standard_pbr_gpu_light_lighting(",
            "zr_environment_pbr_indirect_normalized(",
            "zr_gpu_light_shadow_visibility(",
        ] {
            assert!(
                assembly.wgsl_source.contains(required),
                "Forward PBR must retain its base closure `{required}`"
            );
        }
        validate_material_shader_template_wgsl(&assembly.wgsl_source)
            .expect("specialized Forward PBR WGSL should validate");
    }

    for excluded in [
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "@group(1) @binding(32) var zr_transmission_scene_color_sampler",
        "@group(1) @binding(38) var<uniform> zr_transmission_scene_color_params",
        "fn zr_pbr_rotated_tangent(",
        "fn zr_aniso_ggx(",
        "fn zr_clearcoat_lobe(",
        "fn zr_pbr_advanced_environment(",
        "fn zr_pbr_screen_space_transmission(",
        "if (ZR_FEATURE_PBR_ANISOTROPY)",
        "if (ZR_FEATURE_PBR_CLEARCOAT",
        "if (ZR_FEATURE_PBR_TRANSMISSION",
    ] {
        assert!(
            !basic.wgsl_source.contains(excluded),
            "basic Forward PBR must exclude unreachable advanced source `{excluded}`"
        );
        assert!(
            advanced_variants
                .iter()
                .all(|(_, assembly)| assembly.wgsl_source.contains(excluded)),
            "every advanced Forward PBR variant must retain `{excluded}`"
        );
    }

    for token in ["zr_pbr_extras.wgsl", "zr_shading_standard_pbr.wgsl"] {
        for (label, advanced) in &advanced_variants {
            assert_ne!(
                include_content_hash(&basic, token),
                include_content_hash(advanced, token),
                "basic and {label} Forward PBR must separate `{token}` cache content"
            );
        }
    }
    for (label, advanced) in &advanced_variants {
        assert!(
            basic.wgsl_source.len() < advanced.wgsl_source.len(),
            "basic Forward PBR must compile less source than {label}, basic={} advanced={}",
            basic.wgsl_source.len(),
            advanced.wgsl_source.len(),
        );
    }
}

#[test]
fn custom_surface_without_a_shading_descriptor_keeps_the_full_pbr_closure() {
    let custom = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("custom Forward template assembly");

    for required in [
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "@group(1) @binding(38) var<uniform> zr_transmission_scene_color_params",
        "fn zr_aniso_ggx(",
        "fn zr_clearcoat_lobe(",
        "fn zr_pbr_screen_space_transmission(",
    ] {
        assert!(
            custom.wgsl_source.contains(required),
            "custom surface without an explicit descriptor must retain `{required}`"
        );
    }
    validate_material_shader_template_wgsl(&custom.wgsl_source)
        .expect("custom Forward WGSL should validate with the full PBR closure");
}

#[test]
fn custom_environment_only_surface_keeps_the_generic_forward_closure() {
    let custom = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward).with_features(
            ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        ),
    )
    .expect("custom environment-only Forward template assembly");

    assert_include_token!(custom, "zr_template_forward.wgsl");
    for required in [
        "fn zr_standard_pbr_gpu_light_lighting(",
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "fn zr_aniso_ggx(",
    ] {
        assert!(
            custom.wgsl_source.contains(required),
            "custom environment-only surface must retain generic Forward `{required}`"
        );
    }
    validate_material_shader_template_wgsl(&custom.wgsl_source)
        .expect("custom environment-only Forward WGSL should validate");
}
