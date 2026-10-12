use crate::graphics::scene::scene_renderer::post_process::SMAA_STAGE_FORMAT;

#[test]
fn smaa_stage_textures_store_edge_and_blend_weights() {
    assert_eq!(SMAA_STAGE_FORMAT, wgpu::TextureFormat::Rgba8Unorm);
}
