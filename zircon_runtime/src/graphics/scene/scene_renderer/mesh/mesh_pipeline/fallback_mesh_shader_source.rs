pub(crate) const FALLBACK_MESH_SHADER: &str = concat!(
    include_str!("../shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_lightmap.wgsl"),
    "\n",
    include_str!("../shaders/fallback_mesh.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment.wgsl")
);

#[cfg(test)]
mod tests {
    use super::FALLBACK_MESH_SHADER;

    #[test]
    fn fallback_mesh_shader_declares_gpu_scene_group() {
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(3) @binding(1) var<storage, read> zr_instance_data"));
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(3) @binding(2) var<storage, read> zr_light_data")
        );
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_gpu_scene_light_count() -> u32"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_skinned_joint_count() -> u32"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_previous_skinned_joint_count() -> u32"));
    }

    #[test]
    fn fallback_mesh_shader_reads_gpu_scene_instance_data() {
        assert!(FALLBACK_MESH_SHADER.contains(
            "fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32)"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("let world_from_local = zr_world_from_local(instance_index);"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let motion_params = zr_gpu_scene_motion_params(instance_index);"));
        assert!(FALLBACK_MESH_SHADER.contains("output.tint = zr_gpu_scene_tint(instance_index);"));
        assert!(FALLBACK_MESH_SHADER
            .contains("output.shadow_params = zr_gpu_scene_shadow_params(instance_index);"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains("model_data"));
    }

    #[test]
    fn fallback_mesh_shader_receives_shadow_atlas_resources() {
        for expected in [
            "@group(1) @binding(8) var zr_shadow_atlas: texture_depth_2d;",
            "@group(1) @binding(9) var zr_shadow_sampler: sampler_comparison;",
            "@group(1) @binding(10) var<storage, read> zr_shadow_slots",
            "@group(1) @binding(11) var<uniform> zr_shadow_globals",
            "fn zr_gpu_light_shadow_visibility",
            "fn zr_sample_shadow_slot",
            "textureSampleCompareLevel(zr_shadow_atlas, zr_shadow_sampler, sample_uv, receiver_depth)",
            "fn zr_shadow_slot_pcf_quality",
            "ZR_SHADOW_PCF_QUALITY_MEDIUM",
            "ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS",
            "ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(expected),
                "fallback mesh shader should expose shadow atlas resource `{expected}`"
            );
        }
        assert!(!FALLBACK_MESH_SHADER.contains("struct ShadowReceiverUniform"));
        assert!(!FALLBACK_MESH_SHADER.contains("shadow_map_tex"));
        assert!(!FALLBACK_MESH_SHADER.contains("shadow_compare_sampler"));
        assert!(!FALLBACK_MESH_SHADER.contains("sample_shadow_visibility"));
        assert!(!FALLBACK_MESH_SHADER.contains("world_to_shadow_coord"));
        assert!(!FALLBACK_MESH_SHADER.contains("textureSampleCompare("));
    }

    #[test]
    fn fallback_mesh_shader_receives_light_grid_resources() {
        for expected in [
            "@group(1) @binding(20) var<uniform> zr_light_grid_params",
            "@group(1) @binding(21) var<storage, read> zr_light_zbins",
            "@group(1) @binding(22) var<storage, read> zr_light_tile_masks",
            "fn zr_light_mask_word",
            "fn zr_light_zbin_header",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(expected),
                "fallback mesh shader should declare or use `{expected}` for light-grid lighting"
            );
        }
    }

    #[test]
    fn fallback_mesh_shader_applies_integrated_volumetric_lighting() {
        for expected in [
            "@group(1) @binding(25) var<uniform> zr_volumetric_apply_params",
            "@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;",
            "@group(1) @binding(27) var zr_volumetric_sampler: sampler;",
            "fn zr_volumetric_apply(",
            "zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z)",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(expected),
                "fallback mesh shader should use volumetric contract `{expected}`"
            );
        }
    }

    #[test]
    fn fallback_mesh_shader_consumes_baked_lightmap_or_probe_indirect() {
        for expected in [
            "@group(1) @binding(23) var<storage, read> zr_light_probe_grid",
            "@group(1) @binding(24) var zr_lightmap_atlas: texture_2d_array<f32>;",
            "@group(1) @binding(28) var zr_lightmap_sampler: sampler;",
            "fn zr_lightmap_baked_irradiance(",
            "zr_lightmap_baked_irradiance(",
        ] {
            assert!(FALLBACK_MESH_SHADER.contains(expected));
        }
    }

    #[test]
    fn fallback_mesh_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(FALLBACK_MESH_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(FALLBACK_MESH_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .expect("fallback mesh shader should validate");
    }

    #[test]
    fn fallback_mesh_shader_reads_gpu_light_buffer_for_all_builtin_light_types() {
        assert!(FALLBACK_MESH_SHADER.contains("@location(2) world_position: vec3<f32>"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let ambient = scene.ambient_color.rgb * material.occlusion;"));
        assert!(FALLBACK_MESH_SHADER.contains("fn gpu_light_lighting"));
        assert!(FALLBACK_MESH_SHADER.contains("fn shade_gpu_light_index"));
        assert!(FALLBACK_MESH_SHADER.contains("if (light_index >= zr_gpu_scene_light_count())"));
        assert!(FALLBACK_MESH_SHADER.contains("let light = zr_gpu_light(light_index);"));
        assert!(FALLBACK_MESH_SHADER
            .contains("zr_light_mask_word(tile_base, bin, word, zr_light_grid_params)"));
        assert!(FALLBACK_MESH_SHADER.contains("firstTrailingBit(mask)"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_DIRECTIONAL"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_POINT"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_SPOT"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_RECT"));
        assert!(FALLBACK_MESH_SHADER
            .contains("zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z)"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let direct_lights = gpu_light_lighting(input.clip_position.xy"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_pbr_indirect"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_is_realtime_ibl"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_procedural_sky_color"));
        assert!(FALLBACK_MESH_SHADER.contains("scene.environment_sample_params.x"));
        assert!(FALLBACK_MESH_SHADER.contains("zr_environment_sh9.coefficients[0].rgb"));
        assert!(FALLBACK_MESH_SHADER.contains("override ZR_ENV_DIFFUSE_IEM: bool = false;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(0) @binding(2) var zr_environment_sampler: sampler;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(16) var<storage, read> zr_env_probes"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(17) var<uniform> zr_env_probe_header"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(1) @binding(18) var zr_env_probe_cubemaps: texture_cube_array<f32>;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_box_project("));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_select_probes("));
        assert!(FALLBACK_MESH_SHADER.contains("        input.world_position,"));
        assert!(FALLBACK_MESH_SHADER.contains("textureSampleLevel("));
        assert!(FALLBACK_MESH_SHADER.contains("zr_environment_source_cube"));
        assert!(FALLBACK_MESH_SHADER.contains("zr_environment_specular_pmrem_cube"));
        assert!(FALLBACK_MESH_SHADER.contains("zr_environment_irradiance_cube"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_mip_from_roughness"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_sh9_eval"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_irradiance_cube_color"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_env_brdf_lut"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_env_brdf_approx"));
        assert!(FALLBACK_MESH_SHADER
            .contains("material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID"));
        assert!(
            FALLBACK_MESH_SHADER.contains("let environment_lights = zr_environment_pbr_indirect(")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "let lit = diffuse_color * ambient + direct_lights + environment_lights + baked_indirect;"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains("for (var i = 0u; i < light_count; i = i + 1u)"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_position_range"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_color_intensity"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_params"));
        assert!(!FALLBACK_MESH_SHADER.contains("scene.light_color"));
    }

    #[test]
    fn fallback_mesh_shader_exposes_object_velocity_entries() {
        assert!(FALLBACK_MESH_SHADER.contains("view_proj_unjittered: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("previous_view_proj_unjittered: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("fn vs_velocity_object"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_velocity_object"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_local_position = skin_previous_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("let current_clip = scene.view_proj_unjittered * current_world"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_clip = scene.previous_view_proj_unjittered * previous_world"));
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.x <= 0.5"));
    }

    #[test]
    fn fallback_mesh_shader_exposes_taa_reactive_mask_entry() {
        assert!(FALLBACK_MESH_SHADER.contains("fn sampled_base_color"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_taa_reactive_mask"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_taa_reactive_material_mask"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let authored_strength = clamp(material_properties.data8.x, 0.0, 1.0);"));
        assert!(FALLBACK_MESH_SHADER.contains("let reactive_mask = max(alpha, authored_strength);"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let reactive_mask = clamp(material_properties.data8.x, 0.0, 1.0);"));
        assert!(FALLBACK_MESH_SHADER.contains("return reactive_mask;"));
        assert!(FALLBACK_MESH_SHADER.contains("discard;"));
    }

    #[test]
    fn fallback_mesh_shader_exposes_skinning_vertex_channels() {
        assert!(FALLBACK_MESH_SHADER.contains("@location(3) joint_indices: vec4<u32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(4) joint_weights: vec4<f32>"));
    }

    #[test]
    fn fallback_mesh_shader_consumes_vertex_color_and_exposes_tangent_channel() {
        assert!(FALLBACK_MESH_SHADER.contains("@location(5) tangent: vec4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(6) color: vec4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(7) uv1: vec2<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(3) vertex_color: vec4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(4) world_tangent: vec3<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(5) tangent_handedness: f32"));
        assert!(FALLBACK_MESH_SHADER.contains("output.vertex_color = input.color;"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "textureSample(albedo_tex, albedo_sampler, base_color_uv).rgba * input.tint * input.vertex_color"
        ));
    }

    #[test]
    fn fallback_mesh_shader_samples_normal_map_with_tangent_frame() {
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(3) var normal_tex: texture_2d<f32>;")
        );
        assert!(FALLBACK_MESH_SHADER.contains("@group(2) @binding(4) var normal_sampler: sampler;"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_tangent"));
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.w <= 0.5"));
        assert!(FALLBACK_MESH_SHADER
            .contains("textureSample(normal_tex, normal_sampler, normal_uv).xyz * 2.0"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let normal_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data3, material_properties.data7.y);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("let bitangent = normalize_or_zero(cross(geometric_normal, tangent) * input.tangent_handedness);"));
        assert!(FALLBACK_MESH_SHADER.contains("let world_normal = sampled_world_normal(input);"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let lambert = max(dot(world_normal, light_vector), 0.0);"));
    }

    #[test]
    fn fallback_mesh_shader_samples_standard_pbr_texture_set() {
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(5) var metallic_roughness_tex: texture_2d<f32>;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(6) var metallic_roughness_sampler: sampler;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(7) var occlusion_tex: texture_2d<f32>;"));
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(8) var occlusion_sampler: sampler;")
        );
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(9) var emissive_tex: texture_2d<f32>;"));
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(10) var emissive_sampler: sampler;")
        );
        assert!(FALLBACK_MESH_SHADER.contains("material_properties.data0.x * metallic_roughness.b"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "roughness = clamp(roughness * metallic_roughness.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("let metallic_roughness_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data4, material_properties.data7.z);"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let occlusion_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data5, material_properties.data7.w);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let emissive_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data6, material_properties.data1.w);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "occlusion = clamp(occlusion * textureSample(occlusion_tex, occlusion_sampler, occlusion_uv).r"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "material_properties.data1.rgb, vec3<f32>(0.0, 0.0, 0.0)) * textureSample(emissive_tex, emissive_sampler, emissive_uv).rgb"
        ));
    }

    #[test]
    fn fallback_mesh_shader_dispatches_builtin_shading_models() {
        for expected in [
            "const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;",
            "const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;",
            "const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;",
            "fn decode_shading_model_id(encoded: f32) -> u32",
            "round(clamp(encoded, 0.0, 1.0) * 255.0)",
            "shading_model_id: u32",
            "decode_shading_model_id(material_properties.data8.y)",
            "fn material_diffuse_color(material: SampledMaterial) -> vec3<f32>",
            "material.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID",
            "fn shade_standard_pbr_light_vector",
            "fn shade_blinn_phong_light_vector",
            "return shade_blinn_phong_light_vector(light_vector, radiance, world_normal, material, diffuse_color);",
            "return shade_standard_pbr_light_vector(light_vector, radiance, world_normal, material, diffuse_color);",
            "material.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID",
            "let shaded = material.albedo.rgb + material.emissive;",
            "zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z)",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(expected),
                "fallback mesh shader should use `{expected}` for built-in shading model dispatch"
            );
        }
        assert!(!FALLBACK_MESH_SHADER
            .contains("let shaded = mix(lit, material.albedo.rgb, clamp(material.unlit"));
    }

    #[test]
    fn fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(FALLBACK_MESH_SHADER.contains("struct ZrSkinnedJointPaletteStorage"));
        assert!(FALLBACK_MESH_SHADER.contains("joint_matrices: array<mat4x4<f32>, 256>"));
        assert!(FALLBACK_MESH_SHADER.contains("params: vec4<u32>"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette: ZrSkinnedJointPaletteStorage;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette: ZrSkinnedJointPaletteStorage;"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER
            .contains("fn zr_previous_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>"));
        assert!(!FALLBACK_MESH_SHADER.contains("struct SkinnedJointPaletteUniform"));
        assert!(!FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(1) var<uniform> skinned_joint_palette"));
        assert!(!FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(2) var<uniform> previous_skinned_joint_palette"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_previous_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_normal"));
        assert!(FALLBACK_MESH_SHADER.contains("motion_params.y <= 0.5"));
        assert!(FALLBACK_MESH_SHADER.contains("motion_params.z <= 0.5"));
        assert!(FALLBACK_MESH_SHADER.contains("joint_index >= zr_skinned_joint_count()"));
        assert!(FALLBACK_MESH_SHADER.contains("joint_index >= zr_previous_skinned_joint_count()"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let local_position = skin_vertex_position(input.position, input.joint_indices, input.joint_weights, motion_params);"
        ));
    }
}
