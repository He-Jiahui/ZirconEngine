const COLOR_LUT_BAKE_SHADER: &str = include_str!("../../../../shaders/color_lut_bake.wgsl");

#[test]
fn color_lut_bake_shader_declares_compute_entry_and_3d_storage_output() {
    assert!(COLOR_LUT_BAKE_SHADER.contains("@compute @workgroup_size(4, 4, 4)"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("fn cs_main"));
    assert!(
        COLOR_LUT_BAKE_SHADER.contains("var color_lut_out: texture_storage_3d<rgba16float, write>")
    );
    assert!(COLOR_LUT_BAKE_SHADER.contains("textureStore(color_lut_out"));
}

#[test]
fn color_lut_bake_shader_bakes_tonemap_grading_and_user_lut() {
    assert!(COLOR_LUT_BAKE_SHADER.contains("fn apply_tonemap"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("fn apply_color_grading"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("fn sample_user_lut"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("@binding(1) var<storage, read> exposure_buffer"));
    assert!(COLOR_LUT_BAKE_SHADER
        .contains("let exposure = exp2(params.tonemap_lut.x) * max(exposure_buffer[0].x, 0.0);"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("let exposure = params.grading.x;"));
    assert!(COLOR_LUT_BAKE_SHADER.contains("let lut_intensity = clamp(params.tonemap_lut.z"));
}
