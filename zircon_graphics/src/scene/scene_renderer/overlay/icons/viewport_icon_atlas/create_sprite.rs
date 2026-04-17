use std::sync::Arc;

use super::super::viewport_icon_sprite::ViewportIconSprite;

pub(super) fn create_sprite(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Arc<ViewportIconSprite> {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-viewport-icon-texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-viewport-icon-bind-group"),
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    }));

    Arc::new(ViewportIconSprite {
        _texture: texture,
        _view: view,
        bind_group,
    })
}
