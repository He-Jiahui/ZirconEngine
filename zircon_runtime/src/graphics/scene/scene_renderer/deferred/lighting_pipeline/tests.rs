use crate::core::framework::render::{GBufferChannelMask, ShadingModelDescriptor, ShadingModelId};

use super::shader_source::{
    assemble_deferred_lighting_shader_source, DeferredLightingShaderSourceError,
    DeferredLightingShaderSourceRequest, DEFERRED_LIGHTING_SHADER,
};

mod runtime_pipeline;

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
    let environment_sh9 = scene_uniform.find("environment_sh9").unwrap();

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
            && environment_params < environment_sample_params
            && environment_sample_params < environment_sh9,
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
fn deferred_lighting_shader_applies_environment_reflections_to_standard_pbr() {
    for expected in [
        "// include: zr_environment.wgsl",
        "fn zr_environment_pbr_indirect",
        "scene.sky_horizon_color.rgb",
        "scene.sky_zenith_color.rgb",
        "scene.sky_ground_color.rgb",
        "scene.environment_params.w > 0.5",
        "scene.environment_sample_params.x",
        "scene.environment_sh9[0].rgb",
        "override ZR_ENV_DIFFUSE_IEM: bool = false;",
        "@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;",
        "@group(0) @binding(2) var zr_environment_sampler: sampler;",
        "@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;",
        "@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;",
        "@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;",
        "textureSampleLevel(",
        "zr_environment_source_cube",
        "zr_environment_specular_pmrem_cube",
        "zr_environment_irradiance_cube",
        "fn zr_environment_mip_from_roughness",
        "fn zr_environment_sh9_eval",
        "fn zr_environment_irradiance_cube_color",
        "fn zr_environment_env_brdf_lut",
        "fn zr_environment_env_brdf_approx",
        "fn zr_environment_fix_cube_lookup",
        "return clamp(roughness, 0.0, 1.0) * max(max_mip, 0.0);",
        "zr_environment_fix_cube_lookup(rotated, clamped_lod)",
        "camera_world_position: vec4<f32>",
        "camera_view_direction: vec4<f32>",
        "let view_dir = scene_view_dir_ws(world_position);",
        "let environment_lights = zr_environment_pbr_indirect(",
        "shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID",
        "let color = diffuse_color * ambient + direct_lights + environment_lights;",
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
        "fn shade_standard_pbr_light_vector",
        "fn shade_blinn_phong_light_vector",
        "fn shade_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>, shading_model_id: u32)",
        "if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        "return shade_blinn_phong_light_vector(light_vector, radiance, normal, roughness, diffuse_color, view_dir);",
        "return shade_standard_pbr_light_vector(light_vector, radiance, normal, roughness, metallic, diffuse_color, view_dir);",
        "fn deferred_diffuse_color(albedo: vec4<f32>, metallic: f32, shading_model_id: u32) -> vec3<f32>",
        "fn shade_deferred_lit(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, shading_model_id: u32)",
        "let direct_lights = gpu_light_lighting(position.xy, world_position, normal, roughness, metallic, occlusion, diffuse_color, view_dir, shading_model_id, receive_shadows);",
        "return shade_deferred_unlit(albedo);",
        "return shade_deferred_blinn_phong(position, coord, albedo, material, normal);",
        "return shade_deferred_standard_pbr(position, coord, albedo, material, normal);",
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
    assert!(
        source.contains("return shade_deferred_toon(position, coord, albedo, material, normal);")
    );
    assert!(source.contains(
        "return shade_deferred_standard_pbr(position, coord, albedo, material, normal);"
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
