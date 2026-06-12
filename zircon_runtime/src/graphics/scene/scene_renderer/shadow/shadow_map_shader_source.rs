pub(in crate::graphics::scene::scene_renderer::shadow) const SHADOW_MAP_SHADER: &str = concat!(
    include_str!("../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("shaders/shadow_map.wgsl")
);

#[cfg(test)]
mod tests {
    use super::SHADOW_MAP_SHADER;

    #[test]
    fn shadow_map_shader_declares_gpu_scene_group() {
        assert!(SHADOW_MAP_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data"));
        assert!(
            SHADOW_MAP_SHADER.contains("@group(3) @binding(1) var<storage, read> zr_instance_data")
        );
        assert!(SHADOW_MAP_SHADER
            .contains("@group(3) @binding(3) var<uniform> zr_skinned_joint_palette"));
        assert!(SHADOW_MAP_SHADER
            .contains("@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette"));
        assert!(SHADOW_MAP_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
        assert!(SHADOW_MAP_SHADER.contains("fn zr_skinned_joint_count() -> u32"));
    }

    #[test]
    fn shadow_map_shader_reads_gpu_scene_instance_data() {
        assert!(SHADOW_MAP_SHADER.contains(
            "fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32)"
        ));
        assert!(SHADOW_MAP_SHADER.contains("zr_gpu_scene_motion_params(instance_index)"));
        assert!(SHADOW_MAP_SHADER.contains(
            "let world = zr_world_from_local(instance_index) * vec4<f32>(local_position, 1.0);"
        ));
        assert!(SHADOW_MAP_SHADER.contains("output.tint = zr_gpu_scene_tint(instance_index);"));
        assert!(SHADOW_MAP_SHADER
            .contains("output.shadow_params = zr_gpu_scene_shadow_params(instance_index);"));
        assert!(!SHADOW_MAP_SHADER.contains("model_data"));
    }

    #[test]
    fn shadow_map_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(SHADOW_MAP_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SHADOW_MAP_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .expect("shadow map shader should validate");
    }

    #[test]
    fn shadow_map_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(SHADOW_MAP_SHADER.contains("struct ZrSkinnedJointPaletteUniform"));
        assert!(SHADOW_MAP_SHADER.contains("joint_matrices: array<mat4x4<f32>, 256>"));
        assert!(SHADOW_MAP_SHADER.contains("params: vec4<u32>"));
        assert!(SHADOW_MAP_SHADER.contains(
            "@group(3) @binding(3) var<uniform> zr_skinned_joint_palette: ZrSkinnedJointPaletteUniform;"
        ));
        assert!(SHADOW_MAP_SHADER
            .contains("fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>"));
        assert!(!SHADOW_MAP_SHADER.contains("struct SkinnedJointPaletteUniform"));
        assert!(
            !SHADOW_MAP_SHADER.contains("@group(1) @binding(1) var<uniform> skinned_joint_palette")
        );
        assert!(SHADOW_MAP_SHADER.contains("@location(3) joint_indices: vec4<u32>"));
        assert!(SHADOW_MAP_SHADER.contains("@location(4) joint_weights: vec4<f32>"));
        assert!(SHADOW_MAP_SHADER.contains("fn skin_vertex_position"));
        assert!(SHADOW_MAP_SHADER.contains("motion_params.y <= 0.5"));
        assert!(SHADOW_MAP_SHADER.contains("joint_index >= zr_skinned_joint_count()"));
    }
}
