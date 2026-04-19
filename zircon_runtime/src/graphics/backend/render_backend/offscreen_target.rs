use crate::core::math::UVec2;

pub(crate) struct OffscreenTarget {
    pub(crate) size: UVec2,
    pub(crate) final_color: wgpu::Texture,
    pub(crate) final_color_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(crate) scene_color: wgpu::Texture,
    pub(crate) scene_color_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(crate) bloom: wgpu::Texture,
    pub(crate) bloom_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(crate) gbuffer_albedo: wgpu::Texture,
    pub(crate) gbuffer_albedo_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(crate) normal: wgpu::Texture,
    pub(crate) normal_view: wgpu::TextureView,
    pub(crate) ambient_occlusion: wgpu::Texture,
    pub(crate) ambient_occlusion_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(crate) depth: wgpu::Texture,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) cluster_dimensions: UVec2,
    pub(crate) cluster_buffer: wgpu::Buffer,
    pub(crate) cluster_buffer_bytes: usize,
}
