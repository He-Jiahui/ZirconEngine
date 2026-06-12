use crate::core::math::UVec2;

pub(crate) struct SceneFrameHistoryTextures {
    pub(crate) size: UVec2,
    pub(crate) hzb_furthest_size: UVec2,
    pub(crate) hzb_furthest_mip_count: u32,
    pub(crate) scene_color: wgpu::Texture,
    pub(crate) scene_color_view: wgpu::TextureView,
    pub(crate) global_illumination: wgpu::Texture,
    pub(crate) global_illumination_view: wgpu::TextureView,
    pub(crate) ambient_occlusion: wgpu::Texture,
    pub(crate) ambient_occlusion_view: wgpu::TextureView,
    pub(crate) screen_space_reflection: wgpu::Texture,
    pub(crate) screen_space_reflection_view: wgpu::TextureView,
    pub(crate) hzb_furthest: wgpu::Texture,
    pub(crate) hzb_furthest_view: wgpu::TextureView,
}
