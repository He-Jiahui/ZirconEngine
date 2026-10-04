use super::OUTPUT_TARGET_LINEAR_CONVERSION_SHADER;

#[test]
fn output_target_linear_conversion_shader_loads_source_texel_without_sampler() {
    assert!(OUTPUT_TARGET_LINEAR_CONVERSION_SHADER.contains("texture_2d<f32>"));
    assert!(OUTPUT_TARGET_LINEAR_CONVERSION_SHADER.contains("textureLoad(source_tex"));
    assert!(!OUTPUT_TARGET_LINEAR_CONVERSION_SHADER.contains("sampler"));
}
