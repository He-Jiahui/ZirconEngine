const HZB_BUILD_SHADER: &str = include_str!("../../../../shaders/hzb_build.wgsl");
const HZB_BUILD_MSAA_SHADER: &str = include_str!("../../../../shaders/hzb_build_msaa.wgsl");
const ZR_REDUCE_INCLUDE: &str = include_str!("../../../../../../../shader/includes/zr_reduce.wgsl");

#[test]
fn hzb_shader_declares_reduce_entry_and_storage_target() {
    assert!(HZB_BUILD_SHADER.contains("@compute @workgroup_size(8, 8, 1)"));
    assert!(HZB_BUILD_SHADER.contains("fn cs_main"));
    assert!(HZB_BUILD_SHADER.contains("texture_storage_2d<rgba16float, write>"));
}

#[test]
fn hzb_shader_preserves_furthest_and_closest_depth_per_mip() {
    for source in [HZB_BUILD_SHADER, HZB_BUILD_MSAA_SHADER] {
        assert!(source.contains("struct HzbDepthRange"));
        assert!(source.contains("zr_reduce_max_f32"));
        assert!(source.contains("zr_reduce_min_f32"));
        assert!(source.contains("parent_range.y"));
        assert!(source.contains("depth_range.furthest"));
        assert!(source.contains("depth_range.closest"));
    }
    assert!(ZR_REDUCE_INCLUDE.contains("fn zr_reduce_min_f32"));
    assert!(ZR_REDUCE_INCLUDE.contains("fn zr_reduce_max_f32"));
    assert!(ZR_REDUCE_INCLUDE.contains("fn zr_reduce_add_f32"));
    assert!(HZB_BUILD_MSAA_SHADER.contains("texture_depth_multisampled_2d"));
    assert!(HZB_BUILD_MSAA_SHADER.contains("textureNumSamples(scene_depth_tex)"));
}
