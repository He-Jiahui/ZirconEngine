use crate::core::math::UVec2;

pub(crate) struct SceneFrameHistoryTextures {
    pub(crate) size: UVec2,
    pub(crate) scene_color: wgpu::Texture,
    pub(crate) scene_color_view: wgpu::TextureView,
    pub(crate) global_illumination: wgpu::Texture,
    pub(crate) global_illumination_view: wgpu::TextureView,
    pub(crate) ambient_occlusion: wgpu::Texture,
    pub(crate) ambient_occlusion_view: wgpu::TextureView,
}
