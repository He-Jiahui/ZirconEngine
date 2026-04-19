use super::super::constants::DEPTH_FORMAT;
use super::super::target_extent::texture_extent;

pub(crate) fn create_depth_texture(
    device: &wgpu::Device,
    size: crate::core::math::UVec2,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-depth"),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
