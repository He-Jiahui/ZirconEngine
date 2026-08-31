pub(crate) const FALLBACK_MESH_SHADER: &str = concat!(
    include_str!("../shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_light_cookie.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_irradiance_volume.wgsl"),
    "\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_lightmap.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_pbr_common.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_pbr_extras_core.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_normal.wgsl"),
    "\n",
    include_str!("../shaders/fallback_mesh.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment.wgsl")
);

#[cfg(test)]
mod tests {
    use super::FALLBACK_MESH_SHADER;

    #[test]
    fn fallback_mesh_shader_declares_gpu_scene_group() {
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(3) @binding(1) var<storage, read> zr_instance_data")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(3) @binding(2) var<storage, read> zr_light_data")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_gpu_scene_light_count() -> u32"));
        assert!(
            FALLBACK_MESH_SHADER.contains("fn zr_skinned_joint_count(instance_index: u32) -> u32")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("fn zr_previous_skinned_joint_count(instance_index: u32) -> u32")
        );
    }

    #[test]
    fn fallback_mesh_shader_applies_precomputed_per_slot_uv_rotation() {
        for required in [
            "data13: vec4<f32>,",
            "data14: vec4<f32>,",
            "data15: vec4<f32>,",
            "let scaled = uv * transform.xy;",
            "scaled.x * rotation_sin_cos.x - scaled.y * rotation_sin_cos.y",
            "scaled.x * rotation_sin_cos.y + scaled.y * rotation_sin_cos.x",
            "material_properties.data13.xy",
            "material_properties.data13.zw",
            "material_properties.data14.xy",
            "material_properties.data14.zw",
            "material_properties.data15.xy",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(required),
                "fallback mesh shader must retain `{required}` for KHR_texture_transform rotation"
            );
        }
        assert!(
            !FALLBACK_MESH_SHADER.contains("uv * transform.xy + transform.zw"),
            "fallback mesh shader must not silently drop texture rotation"
        );
    }

    #[test]
    fn fallback_mesh_shader_reads_gpu_scene_instance_data() {
        assert!(FALLBACK_MESH_SHADER.contains(
            "fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32)"
        ));
        assert!(
            FALLBACK_MESH_SHADER.contains("let instance = zr_gpu_scene_instance(instance_index);")
        );
        assert!(FALLBACK_MESH_SHADER.contains("let world_from_local = instance.world_from_local;"));
        assert!(FALLBACK_MESH_SHADER.contains("let instance_flags = instance.flags;"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let motion_params = zr_gpu_scene_motion_params(instance_index);")
        );
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_gpu_scene_normal_to_world_direction("));
        assert!(FALLBACK_MESH_SHADER.contains(
            "output.world_normal = normalize_or_zero(zr_gpu_scene_normal_to_world_direction(world_from_local, instance_flags, local_normal));"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "output.world_tangent = normalize_or_zero(zr_gpu_scene_tangent_to_world_direction(world_from_local, instance_flags, local_tangent));"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "output.tangent_handedness = select(-1.0, 1.0, input.tangent.w >= 0.0) * zr_gpu_scene_tangent_handedness_scale(instance_flags);"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains(
            "output.world_normal = normalize_or_zero((world_from_local * vec4<f32>(local_normal, 0.0)).xyz);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("output.tint = zr_gpu_scene_tint(instance_index);"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("output.shadow_params = zr_gpu_scene_shadow_params(instance_index);")
        );
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
        assert!(FALLBACK_MESH_SHADER.contains("let ambient = zr_scene_ambient_color("));
        assert!(FALLBACK_MESH_SHADER.contains("fn gpu_light_lighting"));
        assert!(FALLBACK_MESH_SHADER.contains("fn shade_gpu_light_index"));
        assert!(FALLBACK_MESH_SHADER.contains("if (light_index >= zr_gpu_scene_light_count())"));
        assert!(FALLBACK_MESH_SHADER.contains("let light = zr_gpu_light(light_index);"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("zr_light_mask_word(tile_base, bin, word, zr_light_grid_params)")
        );
        assert!(FALLBACK_MESH_SHADER.contains("firstTrailingBit(mask)"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_DIRECTIONAL"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_POINT"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_SPOT"));
        assert!(FALLBACK_MESH_SHADER.contains("ZR_GPU_LIGHT_TYPE_RECT"));
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z)"
            )
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let direct_lights = gpu_light_lighting(input.clip_position.xy")
        );
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_pbr_indirect"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_is_realtime_ibl"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_procedural_sky_color"));
        assert!(FALLBACK_MESH_SHADER.contains("scene.environment_sample_params.x"));
        assert!(FALLBACK_MESH_SHADER.contains("zr_environment_sh9.coefficients[0].rgb"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_has_irradiance_cube"));
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;"
            )
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(0) @binding(2) var zr_environment_sampler: sampler;")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;"
        ));
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;"
            )
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(1) @binding(16) var<storage, read> zr_env_probes")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(1) @binding(17) var<uniform> zr_env_probe_header")
        );
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
        assert!(FALLBACK_MESH_SHADER.contains("if (zr_environment_has_irradiance_cube()) {"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_env_brdf_lut"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_environment_env_brdf_approx"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "let environment_lights = zr_environment_pbr_indirect_with_dielectric_f0_normalized("
        ));
        assert!(FALLBACK_MESH_SHADER.contains("material.dielectric_f0,"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let lit = diffuse_color * diffuse_energy_scale * ambient + direct_lights + environment_lights + baked_indirect;"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains("for (var i = 0u; i < light_count; i = i + 1u)"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_position_range"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_color_intensity"));
        assert!(!FALLBACK_MESH_SHADER.contains("point_light_params"));
        assert!(!FALLBACK_MESH_SHADER.contains("scene.light_color"));
    }

    #[test]
    fn fallback_mesh_light_grid_reuses_fragment_normalized_inputs() {
        let light_loop = FALLBACK_MESH_SHADER
            .split("fn gpu_light_lighting(")
            .nth(1)
            .and_then(|source| source.split("fn fs_main(").next())
            .expect("fallback mesh shader should retain the light-grid owner");
        for expected in [
            "world_normal_normalized: vec3<f32>",
            "view_dir_normalized: vec3<f32>",
            "let normalized_world_normal = world_normal_normalized;",
            "let world_view = view_dir_normalized;",
            "if (material.shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID)",
            "direct_f0 = fallback_pbr_material_f0(",
            "normalized_world_normal,",
            "world_view,",
            "direct_f0,",
        ] {
            assert!(
                light_loop.contains(expected),
                "fallback light-grid setup must retain `{expected}`"
            );
        }
        assert!(
            !light_loop.contains("normalize_or_zero("),
            "fallback light-grid setup must reuse fragment-normalized normal and view inputs"
        );

        let surface_normal = FALLBACK_MESH_SHADER
            .split("fn sampled_world_normal(")
            .nth(1)
            .and_then(|source| source.split("fn sampled_base_color(").next())
            .expect("fallback shader should retain the sampled-normal owner");
        for expected in [
            "let geometric_normal = input.world_normal;",
            "let world_normal = tangent * tangent_normal.x + bitangent * tangent_normal.y + geometric_normal * tangent_normal.z;",
            "return normalize(world_normal);",
        ] {
            assert!(
                surface_normal.contains(expected),
                "fallback sampled-normal owner must retain `{expected}` before direct-light reuse"
            );
        }
        let fragment = FALLBACK_MESH_SHADER
            .split("fn fs_main(")
            .nth(1)
            .and_then(|source| source.split("fn fs_taa_reactive_mask(").next())
            .expect("fallback shader should retain the lit fragment owner");
        for expected in [
            "let world_normal = sampled_world_normal(input);",
            "let view_dir = zr_pbr_view_direction_ws(input.world_position);",
            "let direct_lights = gpu_light_lighting(input.clip_position.xy, input.world_position, world_normal, material, diffuse_color, input.shadow_params, view_dir);",
        ] {
            assert!(
                fragment.contains(expected),
                "fallback lit fragment must pass `{expected}` to the direct-light loop"
            );
        }

        let per_light = FALLBACK_MESH_SHADER
            .split("fn shade_gpu_light_index(")
            .nth(1)
            .and_then(|source| source.split("fn gpu_light_lighting(").next())
            .expect("fallback mesh shader should retain the per-light owner");
        assert!(
            per_light.contains("shade_light_vector_normalized("),
            "fallback per-light shading must use normalized inputs from the light-grid owner"
        );
        assert!(
            !per_light.contains("normalize_or_zero(world_normal)"),
            "fallback per-light shading must not renormalize the surface normal"
        );
        assert!(
            !per_light.contains("normalize_or_zero(view_dir)"),
            "fallback per-light shading must not renormalize the view direction"
        );
        for expected in ["direct_f0", "direct_diffuse_brdf"] {
            assert!(
                per_light.contains(expected),
                "fallback per-light shading must consume the precomputed `{expected}`"
            );
        }

        let standard_lobe = FALLBACK_MESH_SHADER
            .split("fn shade_standard_pbr_light_vector_normalized(")
            .nth(1)
            .and_then(|source| source.split("fn shade_blinn_phong_light_vector(").next())
            .expect("fallback mesh shader should retain the standard PBR lobe owner");
        assert!(
            !standard_lobe.contains("let f0 = mix("),
            "fallback standard PBR must not recompute F0 per light"
        );
        assert!(standard_lobe.contains("direct_diffuse_brdf"));
        assert!(!standard_lobe.contains("fallback_pbr_diffuse_energy_scale"));
    }

    #[test]
    fn fallback_mesh_shader_exposes_object_velocity_entries() {
        assert!(FALLBACK_MESH_SHADER.contains("view_proj_unjittered: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("previous_view_proj_unjittered: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("fn vs_velocity_object"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_velocity_object"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let previous_local_position = skin_previous_vertex_position")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);"
        ));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let current_clip = scene.view_proj_unjittered * current_world")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "let previous_clip = scene.previous_view_proj_unjittered * previous_world"
            )
        );
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.x <= 0.5"));
    }

    #[test]
    fn fallback_mesh_scene_uniform_appends_environment_rotation_tail() {
        let scene_uniform = FALLBACK_MESH_SHADER
            .split("struct SceneUniform {")
            .nth(1)
            .and_then(|source| source.split("};").next())
            .expect("fallback mesh shader must declare SceneUniform");

        assert!(
            scene_uniform.contains("environment_rotation_sin_cos: vec4<f32>,"),
            "fallback mesh SceneUniform must mirror the rotation tail"
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
    fn fallback_mesh_shader_exposes_taa_reactive_mask_entry() {
        assert!(FALLBACK_MESH_SHADER.contains("fn sampled_base_color"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_taa_reactive_mask"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_taa_reactive_material_mask"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let authored_strength = clamp(material_properties.data8.x, 0.0, 1.0);")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("let reactive_mask = max(alpha, authored_strength);")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let reactive_mask = clamp(material_properties.data8.x, 0.0, 1.0);")
        );
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
            "textureSampleBias(albedo_tex, albedo_sampler, base_color_uv, scene.camera_world_position.w).rgba * input.tint * input.vertex_color"
        ));
    }

    #[test]
    fn fallback_mesh_shader_shares_normal_map_decoding_with_standard_materials() {
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(3) var normal_tex: texture_2d<f32>;")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(4) var normal_sampler: sampler;")
        );
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_tangent"));
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.w <= 0.5"));
        assert!(FALLBACK_MESH_SHADER.contains("let encoded_normal_xy = textureSampleBias("));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("fn zr_reconstruct_bc5_normal(encoded_xy: vec2<f32>) -> vec3<f32>")
        );
        assert!(FALLBACK_MESH_SHADER.contains("zr_reconstruct_bc5_normal("));
        assert!(!FALLBACK_MESH_SHADER.contains("ZR_NORMAL_CONVENTION_"));
        assert!(FALLBACK_MESH_SHADER.contains("sampled_normal.xy * material_properties.data12.x,"));
        assert!(FALLBACK_MESH_SHADER.contains("material_properties.data13.zw,"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let bitangent = cross(geometric_normal, tangent) * input.tangent_handedness;"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains("input.world_tangent - geometric_normal * dot("));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let sampled_normal = sampled_world_normal(input, uv_mask);")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("return u32(round(clamp(material_properties.data7.x, 0.0, 63.0)));")
        );
        assert!(FALLBACK_MESH_SHADER.contains("(uv_mask & (1u << slot)) != 0u"));
        assert!(
            FALLBACK_MESH_SHADER
                .contains("let lambert = max(dot(world_normal, light_vector), 0.0);")
        );
    }

    #[test]
    fn fallback_mesh_shader_orients_double_sided_normals_from_raster_facing() {
        assert!(FALLBACK_MESH_SHADER.contains("@builtin(front_facing) front_facing: bool"));
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "let world_normal = select(-sampled_normal, sampled_normal, front_facing);"
            )
        );
    }

    #[test]
    fn render_perf_fallback_material_applies_global_mip_bias_to_every_texture_sample() {
        assert_eq!(
            FALLBACK_MESH_SHADER.matches("textureSampleBias(").count(),
            6
        );
        assert!(!FALLBACK_MESH_SHADER.contains("textureSample("));
    }

    #[test]
    fn fallback_mesh_shader_samples_standard_pbr_texture_set() {
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(2) @binding(5) var metallic_roughness_tex: texture_2d<f32>;")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(2) @binding(6) var metallic_roughness_sampler: sampler;")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(2) @binding(7) var occlusion_tex: texture_2d<f32>;")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(8) var occlusion_sampler: sampler;")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("@group(2) @binding(9) var emissive_tex: texture_2d<f32>;")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(10) var emissive_sampler: sampler;")
        );
        assert!(
            FALLBACK_MESH_SHADER.contains("material_properties.data0.x * metallic_roughness.b")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "let roughness = clamp(\n        material_properties.data0.y * metallic_roughness.g,\n        ZR_STANDARD_MATERIAL_MIN_ROUGHNESS,\n        1.0,\n    );"
        ));
        for projection in [
            "let base_color_uv = transform_material_uv_channel(",
            "let metallic_roughness_uv = transform_material_uv_channel(",
            "let occlusion_uv = transform_material_uv_channel(",
            "let emissive_uv = transform_material_uv_channel(",
            "uv_mask,\n        0u,",
            "uv_mask,\n        2u,",
            "uv_mask,\n        3u,",
            "uv_mask,\n        4u,",
        ] {
            assert!(FALLBACK_MESH_SHADER.contains(projection));
        }
        assert!(FALLBACK_MESH_SHADER.contains(
            "let occlusion_sample = textureSampleBias(occlusion_tex, occlusion_sampler, occlusion_uv, scene.camera_world_position.w).r;"
        ));
        assert!(
            FALLBACK_MESH_SHADER.contains(
                "mix(1.0, occlusion_sample, clamp(material_properties.data0.z, 0.0, 1.0))"
            )
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "max(material_properties.data1.rgb, vec3<f32>(0.0, 0.0, 0.0)) * textureSampleBias(emissive_tex, emissive_sampler, emissive_uv, scene.camera_world_position.w).rgb"
        ));
    }

    #[test]
    fn fallback_mesh_shader_preserves_explicit_zero_roughness_factor() {
        let sampled_material = FALLBACK_MESH_SHADER
            .split("fn sampled_material(")
            .nth(1)
            .and_then(|source| source.split("fn light_radiance(").next())
            .expect("fallback mesh shader should retain sampled material owner");

        assert!(sampled_material.contains(
            "material_properties.data0.y * metallic_roughness.g,\n        ZR_STANDARD_MATERIAL_MIN_ROUGHNESS"
        ));
        assert!(
            !sampled_material.contains("select(1.0, material_properties.data0.y"),
            "an explicit glTF roughnessFactor=0 must clamp to the Standard PBR minimum, not fall back to 1.0"
        );
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
            "fn shade_standard_pbr_light_vector_normalized",
            "fn shade_blinn_phong_light_vector",
            "return shade_blinn_phong_light_vector(light_vector, radiance, normalized_world_normal, material, diffuse_color, world_view);",
            "return shade_standard_pbr_light_vector_normalized(light_vector, radiance, normalized_world_normal, material, diffuse_color, direct_f0, world_view);",
            "material.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID",
            "let shaded = material.albedo.rgb + material.emissive;",
            "zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z)",
        ] {
            assert!(
                FALLBACK_MESH_SHADER.contains(expected),
                "fallback mesh shader should use `{expected}` for built-in shading model dispatch"
            );
        }
        assert!(
            !FALLBACK_MESH_SHADER
                .contains("let shaded = mix(lit, material.albedo.rgb, clamp(material.unlit")
        );
    }

    #[test]
    fn fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(FALLBACK_MESH_SHADER.contains("skinning_palette_params: vec4<u32>"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette: array<mat4x4<f32>>;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette: array<mat4x4<f32>>;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "fn zr_skinned_joint_matrix(instance_index: u32, joint_index: u32) -> mat4x4<f32>"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "fn zr_previous_skinned_joint_matrix(instance_index: u32, joint_index: u32) -> mat4x4<f32>"
        ));
        assert!(!FALLBACK_MESH_SHADER.contains("struct ZrSkinnedJointPaletteStorage"));
        assert!(!FALLBACK_MESH_SHADER.contains("struct SkinnedJointPaletteUniform"));
        assert!(
            !FALLBACK_MESH_SHADER
                .contains("@group(1) @binding(1) var<uniform> skinned_joint_palette")
        );
        assert!(
            !FALLBACK_MESH_SHADER
                .contains("@group(1) @binding(2) var<uniform> previous_skinned_joint_palette")
        );
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_previous_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_normal"));
        assert!(FALLBACK_MESH_SHADER.contains("motion_params.y <= 0.5"));
        assert!(FALLBACK_MESH_SHADER.contains("motion_params.z <= 0.5"));
        assert!(
            FALLBACK_MESH_SHADER.contains("joint_index >= zr_skinned_joint_count(instance_index)")
        );
        assert!(
            FALLBACK_MESH_SHADER
                .contains("joint_index >= zr_previous_skinned_joint_count(instance_index)")
        );
        assert!(FALLBACK_MESH_SHADER.contains(
            "let local_position = skin_vertex_position(instance_index, input.position, input.joint_indices, input.joint_weights, motion_params);"
        ));
    }
}

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
    let fallback_body = include_str!("../shaders/fallback_mesh.wgsl");
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
