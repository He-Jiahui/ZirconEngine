use crate::core::framework::render::{
    wgsl_include_paths, GBufferChannelMask, ShadingModelDescriptor, ShadingModelId,
};

use super::shader_source::{
    assemble_deferred_lighting_shader_source, DeferredLightingShaderSourceError,
    DeferredLightingShaderSourceRequest, DEFERRED_LIGHTING_SHADER,
};

mod runtime_pipeline;

#[test]
fn deferred_lighting_shader_applies_integrated_volumetric_lighting() {
    for expected in [
        "@group(1) @binding(25) var<uniform> zr_volumetric_apply_params",
        "@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;",
        "@group(1) @binding(27) var zr_volumetric_sampler: sampler;",
        "fn zr_volumetric_apply(",
        "zr_volumetric_apply(shaded.rgb, position.xy, depth)",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use volumetric contract `{expected}`"
        );
    }
}

#[test]
fn deferred_direct_lighting_reuses_fragment_normalized_inputs() {
    for (label, signature, next_signature) in [
        (
            "light accumulation",
            "fn gpu_light_lighting(",
            "fn gpu_light_lighting_components(",
        ),
        (
            "component accumulation",
            "fn gpu_light_lighting_components(",
            "fn deferred_diffuse_color(",
        ),
    ] {
        let light_loop = DEFERRED_LIGHTING_SHADER
            .split(signature)
            .nth(1)
            .and_then(|source| source.split(next_signature).next())
            .expect("deferred lighting shader should retain the light-grid owner");
        for expected in [
            "normal_normalized: vec3<f32>",
            "view_dir_normalized: vec3<f32>",
            "let world_normal = normal_normalized;",
            "let world_view = view_dir_normalized;",
            "let direct_metallic = clamp(metallic, 0.0, 1.0);",
            "world_normal,",
            "world_view,",
            "direct_f0,",
            "direct_diffuse_brdf,",
        ] {
            assert!(
                light_loop.contains(expected),
                "{label} must retain `{expected}`"
            );
        }
        assert!(
            !light_loop.contains("normalize_or_zero("),
            "{label} must reuse the fragment's zero-safe normalized normal and view direction"
        );
    }

    for (label, signature, direct_call) in [
        (
            "lit deferred path",
            "fn shade_deferred_lit(",
            "gpu_light_lighting(position.xy, world_position, normal, roughness, metallic, diffuse_color, view_dir, shading_model_id, receive_shadows)",
        ),
        (
            "subsurface deferred path",
            "fn shade_deferred_subsurface_components(",
            "gpu_light_lighting_components(position.xy, world_position, normal, roughness, metallic, diffuse_color, view_dir, receive_shadows)",
        ),
    ] {
        let shading = DEFERRED_LIGHTING_SHADER
            .split(signature)
            .nth(1)
            .and_then(|source| source.split("fn shade_deferred_pixel(").next())
            .expect("deferred lighting shader should retain the shading owner");
        assert!(
            shading.contains("let view_dir = scene_view_dir_ws(world_position);"),
            "{label} must use the zero-safe normalized scene view direction"
        );
        assert!(
            shading.contains(direct_call),
            "{label} must pass its fragment-normalized inputs directly to the light loop"
        );
    }
    for expected in [
        "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));",
        "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred fragment entry points must normalize GBuffer normals before direct lighting"
        );
    }

    let light_loop = DEFERRED_LIGHTING_SHADER
        .split("fn gpu_light_lighting(")
        .nth(1)
        .and_then(|source| source.split("fn gpu_light_lighting_components(").next())
        .expect("deferred lighting shader should retain the direct light-grid owner");
    for expected in [
        "if (shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "let direct_metallic = clamp(metallic, 0.0, 1.0);",
        "direct_f0 = mix(",
        "direct_diffuse_brdf =",
    ] {
        assert!(
            light_loop.contains(expected),
            "the deferred Blinn path must bypass `{expected}`"
        );
    }

    let component_loop = DEFERRED_LIGHTING_SHADER
        .split("fn gpu_light_lighting_components(")
        .nth(1)
        .and_then(|source| source.split("fn deferred_diffuse_color(").next())
        .expect("deferred lighting shader should retain the component light-grid owner");
    for expected in [
        "let direct_metallic = clamp(metallic, 0.0, 1.0);",
        "let direct_f0 = mix(",
        "let direct_diffuse_brdf =",
    ] {
        assert!(
            component_loop.contains(expected),
            "the Standard PBR component path must retain `{expected}`"
        );
    }

    let per_light = DEFERRED_LIGHTING_SHADER
        .split("fn shade_gpu_light_index(")
        .nth(1)
        .and_then(|source| source.split("fn shade_gpu_light_index_components(").next())
        .expect("deferred lighting shader should retain the per-light owner");
    assert!(
        per_light.contains("shade_light_vector_normalized("),
        "per-light shading must use normalized inputs supplied by the light-grid owner"
    );
    assert!(
        !per_light.contains("normalize_or_zero(normal)"),
        "per-light shading must not renormalize the surface normal"
    );
    assert!(
        !per_light.contains("normalize_or_zero(view_dir)"),
        "per-light shading must not renormalize the view direction"
    );
    assert!(
        !per_light.contains("metallic: f32"),
        "the deferred per-light owner must not carry metallic after material-factor precomputation"
    );
    for expected in ["direct_f0", "direct_diffuse_brdf"] {
        assert!(
            per_light.contains(expected),
            "per-light shading must consume the precomputed `{expected}`"
        );
    }
    assert!(
        !per_light.contains("occlusion"),
        "deferred direct-light accumulation must not apply ambient occlusion"
    );

    let standard_lobe = DEFERRED_LIGHTING_SHADER
        .split("fn shade_standard_pbr_light_vector_components_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn shade_standard_pbr_light_vector_normalized(")
                .next()
        })
        .expect("deferred lighting must retain the standard PBR lobe owner");
    assert!(
        !standard_lobe.contains("let f0 = mix("),
        "the deferred standard PBR lobe must not recompute F0 per light"
    );
    assert!(
        !standard_lobe.contains("1.0 - clamp(metallic, 0.0, 1.0)"),
        "the deferred standard PBR lobe must not recompute diffuse BRDF per light"
    );

    let per_light_components = DEFERRED_LIGHTING_SHADER
        .split("fn shade_gpu_light_index_components(")
        .nth(1)
        .and_then(|source| source.split("fn gpu_light_lighting(").next())
        .expect("deferred lighting shader should retain the component per-light owner");
    assert!(
        !per_light_components.contains("metallic: f32"),
        "the deferred component per-light owner must not carry metallic after material-factor precomputation"
    );
    assert!(
        !per_light_components.contains("occlusion"),
        "deferred component direct-light accumulation must not apply ambient occlusion"
    );
}

#[test]
fn deferred_lighting_shader_variant_removes_volumetric_bindings_when_disabled() {
    let disabled = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_volumetric_enabled(false),
    )
    .expect("disabled volumetric deferred source");
    let enabled = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_volumetric_enabled(true),
    )
    .expect("enabled volumetric deferred source");

    for binding in ["@binding(25)", "@binding(26)", "@binding(27)"] {
        assert!(!disabled.contains(binding));
        assert!(enabled.contains(binding));
    }
}

#[test]
fn deferred_lighting_shader_accepts_baked_indirect_from_gbuffer_emissive() {
    for expected in [
        "@group(1) @binding(23)",
        "@group(1) @binding(24)",
        "@group(1) @binding(28)",
        "let emissive = textureLoad(gbuffer_emissive_tex, coord, 0).rgb;",
        "add_deferred_emissive(",
    ] {
        assert!(DEFERRED_LIGHTING_SHADER.contains(expected));
    }
}

#[test]
fn deferred_lighting_preserves_frame_clear_for_sky_composition() {
    assert!(DEFERRED_LIGHTING_SHADER.contains("if (albedo.a <= 0.001) {\n        discard;"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("background_tex"));
}

#[test]
fn deferred_lighting_shader_resolves_builtin_dependencies_without_directives() {
    let source =
        assemble_deferred_lighting_shader_source(DeferredLightingShaderSourceRequest::new())
            .expect("builtin deferred shader modules should resolve");
    let irradiance = source
        .find("// include: zr_irradiance_volume.wgsl")
        .expect("irradiance-volume dependency should be assembled");
    let lightmap = source
        .find("// include: zr_lightmap.wgsl")
        .expect("lightmap module should be assembled");

    assert!(irradiance < lightmap);
    assert_eq!(
        source
            .matches("// include: zr_irradiance_volume.wgsl")
            .count(),
        1
    );
    assert!(wgsl_include_paths(&source).is_empty());

    let module = naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("resolved builtin deferred shader should validate");
}

const CUSTOM_TOON_DEFERRED_INCLUDE: &str = r#"
fn shade_deferred_toon(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    let band = clamp(normal.z * 0.5 + 0.5 + material.r * 0.0 + f32(coord.x) * 0.0 + position.x * 0.0, 0.0, 1.0);
    return vec4<f32>(albedo.rgb * band, albedo.a);
}
"#;

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
fn deferred_lighting_shader_matches_scene_uniform_layout() {
    let scene_uniform = DEFERRED_LIGHTING_SHADER
        .split("struct SceneUniform {")
        .nth(1)
        .and_then(|tail| tail.split("};").next())
        .expect("deferred lighting shader should declare SceneUniform");

    let view_proj = scene_uniform.find("view_proj").unwrap();
    let view_proj_unjittered = scene_uniform.find("view_proj_unjittered").unwrap();
    let inverse_view_proj = scene_uniform.find("inverse_view_proj").unwrap();
    let ambient_color = scene_uniform.find("ambient_color").unwrap();
    let previous_view_proj_unjittered =
        scene_uniform.find("previous_view_proj_unjittered").unwrap();
    let motion_params = scene_uniform.find("motion_params").unwrap();
    let jitter_params = scene_uniform.find("jitter_params").unwrap();
    let camera_world_position = scene_uniform.find("camera_world_position").unwrap();
    let camera_view_direction = scene_uniform.find("camera_view_direction").unwrap();
    let sky_horizon_color = scene_uniform.find("sky_horizon_color").unwrap();
    let sky_zenith_color = scene_uniform.find("sky_zenith_color").unwrap();
    let sky_ground_color = scene_uniform.find("sky_ground_color").unwrap();
    let environment_params = scene_uniform.find("environment_params").unwrap();
    let environment_sample_params = scene_uniform.find("environment_sample_params").unwrap();

    assert!(
        view_proj < view_proj_unjittered
            && view_proj_unjittered < inverse_view_proj
            && inverse_view_proj < ambient_color
            && ambient_color < previous_view_proj_unjittered
            && previous_view_proj_unjittered < motion_params
            && motion_params < jitter_params
            && jitter_params < camera_world_position
            && camera_world_position < camera_view_direction
            && camera_view_direction < sky_horizon_color
            && sky_horizon_color < sky_zenith_color
            && sky_zenith_color < sky_ground_color
            && sky_ground_color < environment_params
            && environment_params < environment_sample_params,
        "deferred lighting shader must match the Rust SceneUniform matrix, camera, motion, jitter, and environment layout"
    );
    assert!(!scene_uniform.contains("previous_view_proj:"));
    assert!(!scene_uniform.contains("light_dir"));
    assert!(!scene_uniform.contains("light_color"));
    assert!(!scene_uniform.contains("point_light_position_range"));
}

#[test]
fn deferred_lighting_shader_receives_gpu_light_buffer() {
    for expected in [
        "@group(3) @binding(2) var<storage, read> zr_light_data",
        "fn gpu_light_lighting",
        "fn shade_gpu_light_index",
        "if (light_index >= zr_gpu_scene_light_count())",
        "let light = zr_gpu_light(light_index);",
        "ZR_GPU_LIGHT_TYPE_DIRECTIONAL",
        "ZR_GPU_LIGHT_TYPE_POINT",
        "ZR_GPU_LIGHT_TYPE_SPOT",
        "ZR_GPU_LIGHT_TYPE_RECT",
        "zr_gpu_light_casts_shadow(light)",
        "zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z)",
        "if (receive_shadows)",
        "let direct_lights = gpu_light_lighting(position.xy",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for GPU light-buffer lighting"
        );
    }
    assert!(!DEFERRED_LIGHTING_SHADER.contains("point_light_position_range"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("scene.light_color"));
}

#[test]
fn deferred_lighting_shader_restores_hdr_gbuffer_emissive_for_every_shading_model() {
    for expected in [
        "@group(1) @binding(5) var gbuffer_emissive_tex: texture_2d<f32>;",
        "let emissive = textureLoad(gbuffer_emissive_tex, coord, 0).rgb;",
        "fn add_deferred_emissive(shaded: vec4<f32>, emissive: vec3<f32>)",
        "add_deferred_emissive(shade_deferred_unlit(albedo), emissive)",
        "add_deferred_emissive(\n            shade_deferred_standard_pbr",
        "return apply_deferred_volumetric(",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should restore authored emissive through `{expected}`"
        );
    }
}

#[test]
fn deferred_lighting_shader_applies_environment_reflections_to_standard_pbr() {
    for expected in [
        "// include: zr_environment.wgsl",
        "fn zr_environment_pbr_indirect",
        "fn zr_environment_is_realtime_ibl",
        "fn zr_environment_procedural_sky_color",
        "scene.sky_horizon_color.rgb",
        "scene.sky_zenith_color.rgb",
        "scene.sky_ground_color.rgb",
        "scene.environment_params.w > 0.5",
        "scene.environment_sample_params.x",
        "zr_environment_sh9.coefficients[0].rgb",
        "fn zr_environment_has_irradiance_cube",
        "return scene.environment_params.x > 0.5;",
        "@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;",
        "@group(0) @binding(2) var zr_environment_sampler: sampler;",
        "@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;",
        "@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;",
        "@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;",
        "@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;",
        "@group(1) @binding(16) var<storage, read> zr_env_probes",
        "@group(1) @binding(17) var<uniform> zr_env_probe_header",
        "@group(1) @binding(18) var zr_env_probe_cubemaps: texture_cube_array<f32>;",
        "fn zr_environment_box_project(",
        "fn zr_environment_select_probes(",
        "        world_position,",
        "textureSampleLevel(",
        "zr_environment_source_cube",
        "zr_environment_specular_pmrem_cube",
        "zr_environment_irradiance_cube",
        "fn zr_environment_mip_from_roughness",
        "fn zr_environment_sh9_eval",
        "fn zr_environment_irradiance_cube_color",
        "if (zr_environment_has_irradiance_cube()) {",
        "fn zr_environment_env_brdf_lut",
        "fn zr_environment_env_brdf_approx",
        "fn zr_environment_fix_cube_lookup",
        "return clamp(max_mip - 1.0 + 1.2 * log2(clamped_roughness), 0.0, max_mip);",
        "zr_environment_fix_pmrem_cube_lookup(rotated, clamped_lod)",
        "camera_world_position: vec4<f32>",
        "camera_view_direction: vec4<f32>",
        "let view_dir = scene_view_dir_ws(world_position);",
        "let environment_lights = zr_environment_pbr_indirect_normalized(",
        "shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "let color = diffuse_color * diffuse_energy_scale * ambient + direct_lights + environment_lights;",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for PBR environment reflection"
        );
    }
}

#[test]
fn deferred_lighting_shader_receives_light_grid_resources() {
    for expected in [
        "@group(1) @binding(20) var<uniform> zr_light_grid_params",
        "@group(1) @binding(21) var<storage, read> zr_light_zbins",
        "@group(1) @binding(22) var<storage, read> zr_light_tile_masks",
        "zr_light_mask_word(tile_base, bin, word, zr_light_grid_params)",
        "firstTrailingBit(mask)",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for light-grid lighting"
        );
    }
    assert!(!DEFERRED_LIGHTING_SHADER.contains("for (var i = 0u; i < light_count; i = i + 1u)"));
}

#[test]
fn deferred_lighting_shader_receives_shadow_atlas_resources() {
    for expected in [
        "@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;",
        "@group(1) @binding(8) var zr_shadow_atlas: texture_depth_2d;",
        "@group(1) @binding(9) var zr_shadow_sampler: sampler_comparison;",
        "@group(1) @binding(10) var<storage, read> zr_shadow_slots",
        "@group(1) @binding(11) var<uniform> zr_shadow_globals",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "missing deferred shadow receiver binding `{expected}`"
        );
    }

    for expected in [
        "reconstruct_world_position(coord, depth)",
        "direct_visibility",
        "fn zr_gpu_light_shadow_visibility",
        "fn zr_sample_shadow_slot",
        "textureSampleCompareLevel(zr_shadow_atlas, zr_shadow_sampler, sample_uv, receiver_depth)",
        "fn zr_shadow_slot_pcf_quality",
        "ZR_SHADOW_PCF_QUALITY_MEDIUM",
        "ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS",
        "ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for shadow receiving"
        );
    }
    assert!(!DEFERRED_LIGHTING_SHADER.contains("ShadowReceiverUniform"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("shadow_map_tex"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("shadow_compare_sampler"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("sample_shadow_visibility"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("world_to_shadow_coord"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("textureSampleCompare("));
}

#[test]
fn deferred_lighting_shader_decodes_shading_model_and_receive_shadow_flag_from_gbuffer_material_alpha(
) {
    for expected in [
        "const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;",
        "const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;",
        "const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;",
        "const ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK: u32 = 0x7Fu;",
        "const ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG: u32 = 0x80u;",
        "fn deferred_material_flags(encoded: f32) -> u32",
        "fn decode_shading_model_id(encoded: f32) -> u32",
        "fn decode_receive_shadows(encoded: f32) -> bool",
        "round(clamp(encoded, 0.0, 1.0) * 255.0)",
        "& ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK",
        "& ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG",
        "let shading_model_id = decode_shading_model_id(material.a);",
        "let receive_shadows = decode_receive_shadows(material.a);",
        "fn shade_standard_pbr_light_vector_normalized(",
        "fn shade_blinn_phong_light_vector_normalized(",
        "fn shade_light_vector_normalized(",
        "if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "return shade_blinn_phong_light_vector_normalized(",
        "return shade_standard_pbr_light_vector_normalized(",
        "fn deferred_diffuse_color(albedo: vec4<f32>) -> vec3<f32>",
        "fn shade_deferred_lit(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, shading_model_id: u32)",
        "let direct_lights = gpu_light_lighting(position.xy, world_position, normal, roughness, metallic, diffuse_color, view_dir, shading_model_id, receive_shadows);",
        "if (shading_model_id == ZR_SHADING_MODEL_UNLIT_ID)",
        "if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "shade_deferred_unlit(albedo)",
        "shade_deferred_blinn_phong(position, coord, albedo, material, normal)",
        "shade_deferred_standard_pbr(position, coord, albedo, material, normal)",
        "return apply_deferred_volumetric(",
        "return shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_STANDARD_PBR_ID);",
        "return shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_BLINN_PHONG_ID);",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for shading model dispatch"
        );
    }
}

#[test]
fn deferred_standard_pbr_applies_metallic_diffuse_energy_exactly_once() {
    for expected in [
        "return albedo.rgb;",
        "let direct_metallic = clamp(metallic, 0.0, 1.0);",
        "diffuse_color * (1.0 - direct_metallic) / DEFERRED_PBR_PI",
        "1.0 - metallic,\n        shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "diffuse_color * (1.0 - metallic) * scene.ambient_color.rgb * occlusion",
    ] {
        assert!(DEFERRED_LIGHTING_SHADER.contains(expected));
    }
    assert!(!DEFERRED_LIGHTING_SHADER.contains("mix(1.0, 0.55, metallic)"));
    assert!(!DEFERRED_LIGHTING_SHADER.contains("metallic * 0.45"));
}

#[test]
fn deferred_standard_pbr_direct_lighting_uses_the_same_isotropic_ggx_contract_as_forward() {
    for expected in [
        "fn deferred_pbr_fresnel_schlick(",
        "fn deferred_pbr_smith_visibility(",
        "fn deferred_standard_pbr_isotropic_ggx(",
        "let alpha = max(perceptual_roughness * perceptual_roughness, 0.001);",
        "let alpha_squared = alpha * alpha;",
        "return 0.5 / max(gv + gl, EPSILON);",
        "return deferred_pbr_fresnel_schlick(vo_h, f0) * distribution * visibility;",
        "let direct_metallic = clamp(metallic, 0.0, 1.0);",
        "direct_f0 = mix(vec3<f32>(0.04), max(diffuse_color, vec3<f32>(0.0)), direct_metallic);",
        "diffuse_color * (1.0 - direct_metallic) / DEFERRED_PBR_PI",
        "deferred_standard_pbr_isotropic_ggx(",
        "radiance * specular * lambert,",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred Standard PBR should use forward-compatible GGX direct lighting `{expected}`"
        );
    }
    assert!(
        !DEFERRED_LIGHTING_SHADER.contains("specular_power = mix(96.0, 8.0, roughness)"),
        "the Standard PBR path must not retain the deferred-only Blinn exponent"
    );
}

#[test]
fn deferred_lighting_shader_rejects_unknown_shading_model_deferred_include() {
    let descriptor = toon_shading_model_descriptor();

    let error = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new().with_shading_model_descriptor(descriptor),
    )
    .expect_err("unknown deferred include should fail before shader source assembly succeeds");

    assert_eq!(
        error,
        DeferredLightingShaderSourceError::UnknownDeferredInclude {
            token: "zr_shade_deferred_toon.wgsl".to_string(),
        }
    );
}

#[test]
fn deferred_lighting_shader_uses_custom_shading_model_deferred_include_source() {
    let descriptor = toon_shading_model_descriptor();

    let source = assemble_deferred_lighting_shader_source(
        DeferredLightingShaderSourceRequest::new()
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_deferred_include_source(
                "zr_shade_deferred_toon.wgsl",
                CUSTOM_TOON_DEFERRED_INCLUDE,
            ),
    )
    .expect("custom descriptor-backed deferred lighting shader source assembly");

    assert!(source.contains("// include: zr_shade_deferred_toon.wgsl"));
    assert!(source.contains("fn shade_deferred_toon"));
    assert!(source.contains("if (shading_model_id == 16u)"));
    assert!(source.contains(
        "return apply_deferred_volumetric(add_deferred_emissive(shade_deferred_toon(position, coord, albedo, material, normal), emissive), position, depth);"
    ));
    assert!(source.contains(
        "add_deferred_emissive(\n            shade_deferred_standard_pbr(position, coord, albedo, material, normal),"
    ));

    let module = naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .expect("custom deferred lighting shader should validate");
}

#[test]
fn deferred_lighting_shader_is_valid_wgsl() {
    let module = naga::front::wgsl::parse_str(DEFERRED_LIGHTING_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(DEFERRED_LIGHTING_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .expect("deferred lighting shader should validate");
}
