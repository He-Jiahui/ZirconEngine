pub(in crate::graphics::scene::scene_renderer::mesh) const FALLBACK_MESH_SHADER: &str =
    include_str!("../shaders/fallback_mesh.wgsl");

#[cfg(test)]
mod tests {
    use super::FALLBACK_MESH_SHADER;

    #[test]
    fn fallback_mesh_shader_receives_forward_shadow_map_resources() {
        assert!(FALLBACK_MESH_SHADER.contains("struct ShadowReceiverUniform"));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(4) @binding(0) var shadow_map_tex: texture_depth_2d;"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "@group(4) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;"
        ));
        assert!(FALLBACK_MESH_SHADER
            .contains("@group(4) @binding(2) var shadow_compare_sampler: sampler_comparison;"));
        assert!(FALLBACK_MESH_SHADER.contains("textureSampleCompare("));
        assert!(FALLBACK_MESH_SHADER.contains("if (model_data.shadow_params.z <= 0.5)"));
    }

    #[test]
    fn fallback_mesh_shader_applies_shadow_visibility_to_direct_light_only() {
        assert!(FALLBACK_MESH_SHADER.contains("@location(2) world_position: vec3<f32>"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let direct_visibility = shadow_visibility(input.world_position);"));
        assert!(FALLBACK_MESH_SHADER.contains(
            "scene.ambient_color.rgb + scene.light_color.rgb * lambert * direct_visibility"
        ));
    }

    #[test]
    fn fallback_mesh_shader_exposes_object_motion_vector_entries() {
        assert!(FALLBACK_MESH_SHADER.contains("previous_view_proj: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("previous_model: mat4x4<f32>"));
        assert!(FALLBACK_MESH_SHADER.contains("fn vs_motion_vector"));
        assert!(FALLBACK_MESH_SHADER.contains("fn fs_motion_vector"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_world = model_data.previous_model"));
        assert!(FALLBACK_MESH_SHADER
            .contains("let previous_clip = scene.previous_view_proj * previous_world"));
        assert!(FALLBACK_MESH_SHADER.contains("model_data.motion_params.x <= 0.5"));
    }
}
