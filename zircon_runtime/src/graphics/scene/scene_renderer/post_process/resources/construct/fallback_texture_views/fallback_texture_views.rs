pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) struct FallbackTextureViews
{
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) black_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) white_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hzb_source_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) effect_lut_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) effect_lut_texture_3d_view:
        wgpu::TextureView,
}
