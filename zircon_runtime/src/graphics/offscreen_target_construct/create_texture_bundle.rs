use crate::core::math::UVec2;

use super::texture_bundle::TextureBundle;

pub(super) fn create_texture_bundle(
    device: &wgpu::Device,
    label: &'static str,
    size: UVec2,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> TextureBundle {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    TextureBundle { texture, view }
}
