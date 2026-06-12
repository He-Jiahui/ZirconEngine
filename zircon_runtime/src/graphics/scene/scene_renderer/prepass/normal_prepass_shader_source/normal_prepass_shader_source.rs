pub(in crate::graphics::scene::scene_renderer::prepass) const NORMAL_PREPASS_SHADER: &str = concat!(
    include_str!("../../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../shaders/normal_prepass.wgsl")
);

#[cfg(test)]
mod tests {
    use super::NORMAL_PREPASS_SHADER;

    #[test]
    fn normal_prepass_shader_declares_gpu_scene_group() {
        assert!(NORMAL_PREPASS_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data"));
        assert!(NORMAL_PREPASS_SHADER
            .contains("@group(3) @binding(1) var<storage, read> zr_instance_data"));
        assert!(NORMAL_PREPASS_SHADER
            .contains("@group(3) @binding(3) var<uniform> zr_skinned_joint_palette"));
        assert!(NORMAL_PREPASS_SHADER
            .contains("@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette"));
        assert!(NORMAL_PREPASS_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
        assert!(NORMAL_PREPASS_SHADER.contains("fn zr_skinned_joint_count() -> u32"));
    }

    #[test]
    fn normal_prepass_shader_reads_gpu_scene_instance_data() {
        assert!(NORMAL_PREPASS_SHADER.contains(
            "fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32)"
        ));
        assert!(NORMAL_PREPASS_SHADER
            .contains("let world_from_local = zr_world_from_local(instance_index);"));
        assert!(NORMAL_PREPASS_SHADER
            .contains("let motion_params = zr_gpu_scene_motion_params(instance_index);"));
        assert!(!NORMAL_PREPASS_SHADER.contains("model_data"));
    }

    #[test]
    fn normal_prepass_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(NORMAL_PREPASS_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(NORMAL_PREPASS_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .expect("normal prepass shader should validate");
    }

    #[test]
    fn normal_prepass_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(NORMAL_PREPASS_SHADER.contains("struct ZrSkinnedJointPaletteUniform"));
        assert!(NORMAL_PREPASS_SHADER.contains("joint_matrices: array<mat4x4<f32>, 256>"));
        assert!(NORMAL_PREPASS_SHADER.contains("params: vec4<u32>"));
        assert!(NORMAL_PREPASS_SHADER.contains(
            "@group(3) @binding(3) var<uniform> zr_skinned_joint_palette: ZrSkinnedJointPaletteUniform;"
        ));
        assert!(NORMAL_PREPASS_SHADER
            .contains("fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>"));
        assert!(!NORMAL_PREPASS_SHADER.contains("struct SkinnedJointPaletteUniform"));
        assert!(!NORMAL_PREPASS_SHADER
            .contains("@group(1) @binding(1) var<uniform> skinned_joint_palette"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(3) joint_indices: vec4<u32>"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(4) joint_weights: vec4<f32>"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(5) tangent: vec4<f32>"));
        assert!(NORMAL_PREPASS_SHADER.contains("fn skin_vertex_position"));
        assert!(NORMAL_PREPASS_SHADER.contains("fn skin_vertex_normal"));
        assert!(NORMAL_PREPASS_SHADER.contains("fn skin_vertex_tangent"));
        assert!(NORMAL_PREPASS_SHADER.contains("motion_params.y <= 0.5"));
        assert!(NORMAL_PREPASS_SHADER.contains("joint_index >= zr_skinned_joint_count()"));
    }

    #[test]
    fn normal_prepass_shader_samples_material_normal_map_into_scene_normal() {
        assert!(NORMAL_PREPASS_SHADER
            .contains("@group(2) @binding(2) var normal_tex: texture_2d<f32>;"));
        assert!(
            NORMAL_PREPASS_SHADER.contains("@group(2) @binding(3) var normal_sampler: sampler;")
        );
        assert!(NORMAL_PREPASS_SHADER.contains("@location(1) uv: vec2<f32>"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(2) world_tangent: vec3<f32>"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(3) tangent_handedness: f32"));
        assert!(NORMAL_PREPASS_SHADER.contains("@location(4) uv1: vec2<f32>"));
        assert!(NORMAL_PREPASS_SHADER.contains(
            "@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;"
        ));
        assert!(NORMAL_PREPASS_SHADER.contains("input.motion_params.w <= 0.5"));
        assert!(NORMAL_PREPASS_SHADER.contains(
            "let normal_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data3, material_properties.data7.y);"
        ));
        assert!(NORMAL_PREPASS_SHADER
            .contains("textureSample(normal_tex, normal_sampler, normal_uv).xyz * 2.0"));
        assert!(NORMAL_PREPASS_SHADER.contains("let encoded = sampled_world_normal(input) * 0.5"));
    }
}
