pub(in crate::graphics::scene::scene_renderer::mesh) const FALLBACK_MESH_SHADER: &str = concat!(
    include_str!("../shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../shaders/fallback_mesh.wgsl")
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
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(3) @binding(3) var<uniform> zr_skinned_joint_palette"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette"));
        assert!(FALLBACK_MESH_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
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
    fn fallback_mesh_shader_receives_forward_shadow_map_resources() {
        assert!(FALLBACK_MESH_SHADER.contains("struct ShadowReceiverUniform"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(0) var shadow_map_tex: texture_depth_2d;"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(1) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(1) @binding(2) var shadow_compare_sampler: sampler_comparison;"));
        assert!(FALLBACK_MESH_SHADER.contains("textureSampleCompare("));
        assert!(FALLBACK_MESH_SHADER.contains("if (shadow_params.z <= 0.5)"));
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
    fn fallback_mesh_shader_applies_shadow_visibility_to_directional_light_and_adds_point_lights() {
        assert!(FALLBACK_MESH_SHADER.contains("@location(2) world_position: vec3<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let direct_visibility = shadow_visibility(input.world_position, input.shadow_params);"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("let ambient = scene.ambient_color.rgb * material.occlusion;"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let direct = scene.light_color.rgb * lambert * direct_visibility * material.occlusion;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "scene.light_color.rgb * specular_intensity * direct_visibility * material.occlusion"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("point_light_position_range"));
        assert!(FALLBACK_MESH_SHADER.contains("fn point_light_lighting"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let point_lights = point_light_lighting(input.world_position"));
    }

    #[test]
    fn fallback_mesh_shader_exposes_object_motion_vector_entries() {
        assert!(FALLBACK_MESH_SHADER.contains("previous_view_proj: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("fn vs_motion_vector"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_motion_vector"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_local_position = skin_previous_vertex_position"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_clip = scene.previous_view_proj * previous_world"));
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.x <= 0.5"));
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
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(2) var normal_tex: texture_2d<f32>;")
        );
        assert!(FALLBACK_MESH_SHADER.contains("@group(2) @binding(3) var normal_sampler: sampler;"));
        assert!(FALLBACK_MESH_SHADER.contains("fn skin_vertex_tangent"));
        assert!(FALLBACK_MESH_SHADER.contains("input.motion_params.w <= 0.5"));
        assert!(FALLBACK_MESH_SHADER
            .contains("textureSample(normal_tex, normal_sampler, normal_uv).xyz * 2.0"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "let normal_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data3, material_properties.data7.y);"
        ));
        assert!(FALLBACK_MESH_SHADER.contains("let bitangent = normalize_or_zero(cross(geometric_normal, tangent) * input.tangent_handedness);"));
        assert!(FALLBACK_MESH_SHADER.contains("let world_normal = sampled_world_normal(input);"));
        assert!(
            FALLBACK_MESH_SHADER.contains("let lambert = max(dot(light_dir, world_normal), 0.0);")
        );
    }

    #[test]
    fn fallback_mesh_shader_samples_standard_pbr_texture_set() {
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(4) var metallic_roughness_tex: texture_2d<f32>;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(5) var metallic_roughness_sampler: sampler;"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(6) var occlusion_tex: texture_2d<f32>;"));
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(7) var occlusion_sampler: sampler;")
        );
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(2) @binding(8) var emissive_tex: texture_2d<f32>;"));
        assert!(
            FALLBACK_MESH_SHADER.contains("@group(2) @binding(9) var emissive_sampler: sampler;")
        );
        assert!(FALLBACK_MESH_SHADER.contains("material_properties.data0.x * metallic_roughness.b"));
        assert!(FALLBACK_MESH_SHADER
            .contains("roughness = clamp(roughness * metallic_roughness.g, 0.04, 1.0);"));
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
    fn fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(FALLBACK_MESH_SHADER.contains("struct ZrSkinnedJointPaletteUniform"));
        assert!(FALLBACK_MESH_SHADER.contains("joint_matrices: array<mat4x4<f32>, 256>"));
        assert!(FALLBACK_MESH_SHADER.contains("params: vec4<u32>"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(3) var<uniform> zr_skinned_joint_palette: ZrSkinnedJointPaletteUniform;"
        ));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette: ZrSkinnedJointPaletteUniform;"
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
