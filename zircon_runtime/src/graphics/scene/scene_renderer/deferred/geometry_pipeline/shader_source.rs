pub(in crate::graphics::scene::scene_renderer::deferred) const DEFERRED_GEOMETRY_SHADER: &str = concat!(
    include_str!("../../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../shaders/deferred_geometry.wgsl")
);

#[cfg(test)]
mod tests {
    use super::DEFERRED_GEOMETRY_SHADER;

    #[test]
    fn deferred_geometry_shader_declares_gpu_scene_group() {
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(3) @binding(1) var<storage, read> zr_instance_data"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(3) @binding(3) var<uniform> zr_skinned_joint_palette"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("fn zr_world_from_local(instance_index: u32)"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("fn zr_skinned_joint_count() -> u32"));
    }

    #[test]
    fn deferred_geometry_shader_reads_gpu_scene_instance_data() {
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32)"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("zr_gpu_scene_motion_params(instance_index)"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "let world = zr_world_from_local(instance_index) * vec4<f32>(local_position, 1.0);"
        ));
        assert!(
            DEFERRED_GEOMETRY_SHADER.contains("output.tint = zr_gpu_scene_tint(instance_index);")
        );
        assert!(!DEFERRED_GEOMETRY_SHADER.contains("model_data"));
    }

    #[test]
    fn deferred_geometry_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(DEFERRED_GEOMETRY_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(DEFERRED_GEOMETRY_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .expect("deferred geometry shader should validate");
    }

    #[test]
    fn deferred_geometry_shader_executes_skinned_joint_palette_behind_draw_flag() {
        assert!(DEFERRED_GEOMETRY_SHADER.contains("struct ZrSkinnedJointPaletteUniform"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("joint_matrices: array<mat4x4<f32>, 256>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("params: vec4<u32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "@group(3) @binding(3) var<uniform> zr_skinned_joint_palette: ZrSkinnedJointPaletteUniform;"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>"));
        assert!(!DEFERRED_GEOMETRY_SHADER.contains("struct SkinnedJointPaletteUniform"));
        assert!(!DEFERRED_GEOMETRY_SHADER
            .contains("@group(1) @binding(1) var<uniform> skinned_joint_palette"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(3) joint_indices: vec4<u32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(4) joint_weights: vec4<f32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("fn skin_vertex_position"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("motion_params.y <= 0.5"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("joint_index >= zr_skinned_joint_count()"));
    }

    #[test]
    fn deferred_geometry_shader_consumes_vertex_color_and_exposes_tangent_channel() {
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(5) tangent: vec4<f32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(6) color: vec4<f32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(7) uv1: vec2<f32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("@location(1) vertex_color: vec4<f32>"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains("output.vertex_color = input.color;"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "textureSample(albedo_tex, albedo_sampler, base_color_uv) * input.tint * input.vertex_color"
        ));
    }

    #[test]
    fn deferred_geometry_shader_writes_sampled_material_gbuffer_channels() {
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(2) @binding(4) var metallic_roughness_tex: texture_2d<f32>;"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(2) @binding(5) var metallic_roughness_sampler: sampler;"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(2) @binding(6) var occlusion_tex: texture_2d<f32>;"));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("@group(2) @binding(7) var occlusion_sampler: sampler;"));
        assert!(
            DEFERRED_GEOMETRY_SHADER.contains("material_properties.data0.x * metallic_roughness.b")
        );
        assert!(DEFERRED_GEOMETRY_SHADER.contains("roughness = roughness * metallic_roughness.g;"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER
            .contains("let metallic_roughness_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data4, material_properties.data7.z);"));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "let occlusion_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data5, material_properties.data7.w);"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "occlusion = occlusion * textureSample(occlusion_tex, occlusion_sampler, occlusion_uv).r;"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "let shading_model_id = select(decode_shading_model_id(material_properties.data8.y), ZR_SHADING_MODEL_UNLIT_ID, material_properties.data0.w >= 0.5);"
        ));
        assert!(DEFERRED_GEOMETRY_SHADER.contains(
            "vec4<f32>(metallic, clamp(roughness, 0.04, 1.0), clamp(occlusion, 0.0, 1.0), encode_shading_model_id(shading_model_id))"
        ));
    }

    #[test]
    fn deferred_geometry_shader_encodes_shading_model_id_into_gbuffer_material_alpha() {
        for expected in [
            "const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;",
            "const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;",
            "fn encode_shading_model_id(id: u32) -> f32",
            "fn decode_shading_model_id(encoded: f32) -> u32",
            "return f32(id) / 255.0;",
            "round(clamp(encoded, 0.0, 1.0) * 255.0)",
            "material_properties.data8.y",
            "material_properties.data0.w >= 0.5",
            "encode_shading_model_id(shading_model_id)",
        ] {
            assert!(
                DEFERRED_GEOMETRY_SHADER.contains(expected),
                "deferred geometry shader should use `{expected}` for shading-model G-buffer packing"
            );
        }
    }
}
