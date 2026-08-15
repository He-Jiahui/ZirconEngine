use std::sync::Arc;

use super::super::WgpuUiExternalImage;
use super::UI_IMAGE_TEXTURE_FORMAT;

pub(in super::super) struct WgpuUiImageResource {
    pub(super) texture: wgpu::Texture,
    pub(in super::super) bind_group: wgpu::BindGroup,
    pub(super) size: (u32, u32),
    pub(super) byte_size: u64,
    // Canonical premultiplied-alpha GPU source. Conversion happens once at cache admission.
    pub(super) cpu_rgba: Arc<[u8]>,
    pub(super) is_external: bool,
    pub(super) last_touched_present: u64,
    pub(super) last_uploaded_generation: Option<u64>,
}

impl WgpuUiImageResource {
    pub(super) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
        byte_size: u64,
        cpu_rgba: Arc<[u8]>,
        last_touched_present: u64,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-image"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: UI_IMAGE_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-image-bind-group"),
            layout,
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
        });
        Self {
            texture,
            bind_group,
            size,
            byte_size,
            cpu_rgba,
            is_external: false,
            last_touched_present,
            last_uploaded_generation: None,
        }
    }

    pub(super) fn from_external(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        image: WgpuUiExternalImage,
        byte_size: u64,
        last_touched_present: u64,
    ) -> Self {
        let view = image.create_sample_view();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-external-image-bind-group"),
            layout,
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
        });
        Self {
            texture: image.texture,
            bind_group,
            size: (image.width, image.height),
            // This clone keeps the producer texture resident, so enforce the same GPU budget as
            // ordinary cached images while reporting zero retained CPU pixels.
            byte_size,
            cpu_rgba: Arc::from([]),
            is_external: true,
            last_touched_present,
            last_uploaded_generation: Some(image.generation),
        }
    }
}
