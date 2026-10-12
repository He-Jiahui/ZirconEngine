/// 完整后处理 bind group 的中性纹理集合；各纹理格式/维度与对应 WGSL 槽位匹配。
/// 由系统纹理租约生成，避免效果关闭或历史缺失时出现未绑定资源。
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
