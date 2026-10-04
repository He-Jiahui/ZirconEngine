#[test]
fn fallback_mesh_blinn_phong_direct_lighting_uses_camera_relative_view() {
    let blinn_phong = FALLBACK_MESH_SHADER
        .split("fn shade_blinn_phong_light_vector")
        .nth(1)
        .and_then(|source| source.split("fn shade_light_vector_normalized").next())
        .expect("fallback shader should retain a separate Blinn-Phong light path");

    for expected in [
        "world_view: vec3<f32>",
        "let half_dir = normalize_or_zero(light_vector + world_view);",
    ] {
        assert!(
            blinn_phong.contains(expected),
            "fallback Blinn-Phong direct lighting should use camera-relative `{expected}`"
        );
    }
    assert!(
        !blinn_phong.contains("light_vector + vec3<f32>(0.0, 0.0, 1.0)"),
        "fallback Blinn-Phong direct lighting must not synthesize a fixed +Z camera direction"
    );
}

#[test]
fn fallback_mesh_standard_pbr_uses_source_independent_metallic_diffuse_energy() {
    for expected in [
        "fn material_diffuse_color(material: SampledMaterial) -> vec3<f32>",
        "material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "return zr_pbr_base_color(material.albedo.rgb);",
        "return material.albedo.rgb;",
        "let metallic = clamp(material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);",
        "fn zr_surface_metallic_diffuse_energy_scale(",
        "let specular = zr_pbr_isotropic_ggx(",
        "return (direct_diffuse_brdf + specular) * radiance * lambert;",
        "material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "zr_surface_metallic_diffuse_energy_scale(material.metallic)",
        "let baked_indirect = diffuse_color * diffuse_energy_scale * material.occlusion",
    ] {
        assert!(FALLBACK_MESH_SHADER.contains(expected));
    }
    assert!(!FALLBACK_MESH_SHADER.contains("metallic * 0.45"));
}

#[test]
fn fallback_mesh_standard_pbr_direct_lighting_uses_camera_relative_isotropic_ggx() {
    for expected in [
        "fn zr_pbr_fresnel_schlick(",
        "fn zr_pbr_normalize_or_zero(",
        "fn zr_pbr_smith_joint_visibility_approx(",
        "fn zr_pbr_isotropic_ggx(",
        "let alpha = max(perceptual_roughness * perceptual_roughness, 0.001);",
        "let alpha_squared = alpha * alpha;",
        "return 0.5 / max(visibility_v + visibility_l, ZR_PBR_EXTRAS_EPSILON);",
        "let view_dir = zr_pbr_view_direction_ws(input.world_position);",
        "var direct_f0 = vec3<f32>(0.0);",
        "direct_f0 = fallback_pbr_material_f0(",
        "let specular = zr_pbr_isotropic_ggx(",
        "zr_surface_metallic_diffuse_energy_scale(",
        "return (direct_diffuse_brdf + specular) * radiance * lambert;",
    ] {
        assert!(
            FALLBACK_MESH_SHADER.contains(expected),
            "fallback Standard PBR should use camera-relative GGX direct lighting `{expected}`"
        );
    }
    let fallback_body = include_str!("../../shaders/fallback_mesh.wgsl");
    for rejected in [
        "struct ZrPbrSpecularComponents",
        "fn zr_pbr_isotropic_ggx_components(",
        "specular_components.fresnel",
    ] {
        assert!(!FALLBACK_MESH_SHADER.contains(rejected));
    }
    for duplicate_owner in [
        "struct FallbackPbrGgxTerms",
        "fn fallback_pbr_smith_visibility(",
        "fn fallback_standard_pbr_isotropic_ggx_terms(",
        "fn fallback_standard_pbr_isotropic_ggx(",
        "fn fallback_pbr_diffuse_energy_scale(",
        "struct ZrPbrGgxTerms",
        "fn zr_pbr_isotropic_ggx_terms(",
    ] {
        assert!(
            !fallback_body.contains(duplicate_owner),
            "fallback mesh body must not duplicate shared PBR owner `{duplicate_owner}`"
        );
    }
    assert!(
        !FALLBACK_MESH_SHADER.contains("specular_power = mix(64.0, 4.0, material.roughness)"),
        "fallback Standard PBR must not retain its Blinn exponent"
    );
    let standard_pbr = FALLBACK_MESH_SHADER
        .split("fn shade_standard_pbr_light_vector")
        .nth(1)
        .and_then(|source| source.split("fn shade_blinn_phong_light_vector").next())
        .expect("fallback shader should retain separate Standard PBR and Blinn light paths");
    assert!(
        !standard_pbr.contains("light_vector + vec3<f32>(0.0, 0.0, 1.0)"),
        "fallback Standard PBR must not synthesize a fixed +Z camera direction"
    );
    let per_light = FALLBACK_MESH_SHADER
        .split("fn shade_gpu_light_index(")
        .nth(1)
        .and_then(|source| source.split("fn gpu_light_lighting(").next())
        .expect("fallback shader should retain the per-light light-grid owner");
    assert!(
        !per_light.contains("material.occlusion"),
        "fallback direct-light radiance must not apply ambient occlusion"
    );
}

#[test]
fn fallback_mesh_selects_ambient_by_instance_lightmap_presence() {
    assert!(FALLBACK_MESH_SHADER.contains("lightmapped_ambient_color: vec4<f32>"));
    assert!(FALLBACK_MESH_SHADER.contains("zr_scene_ambient_color("));
    assert!(FALLBACK_MESH_SHADER.contains("zr_gpu_scene_has_lightmap(input.instance_index)"));
}
