use crate::core::math::UVec2;

use super::super::texture_extent::texture_extent;

pub(super) struct GlobalIlluminationHistory {
    lighting: wgpu::Texture,
    lighting_view: wgpu::TextureView,
    temporal_metadata: wgpu::Texture,
    temporal_metadata_view: wgpu::TextureView,
}

impl GlobalIlluminationHistory {
    pub(super) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        let lighting = create_texture(
            device,
            "zircon-history-global-illumination",
            size,
            super::super::super::core::SCENE_COLOR_HDR_FORMAT,
        );
        let lighting_view = lighting.create_view(&wgpu::TextureViewDescriptor::default());
        let temporal_metadata = create_texture(
            device,
            "zircon-history-global-illumination-temporal-metadata",
            size,
            wgpu::TextureFormat::Rgba16Float,
        );
        let temporal_metadata_view =
            temporal_metadata.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            lighting,
            lighting_view,
            temporal_metadata,
            temporal_metadata_view,
        }
    }

    pub(super) const fn lighting(&self) -> &wgpu::Texture {
        &self.lighting
    }

    pub(super) const fn lighting_view(&self) -> &wgpu::TextureView {
        &self.lighting_view
    }

    pub(super) const fn temporal_metadata(&self) -> &wgpu::Texture {
        &self.temporal_metadata
    }

    pub(super) const fn temporal_metadata_view(&self) -> &wgpu::TextureView {
        &self.temporal_metadata_view
    }
}

fn create_texture(
    device: &wgpu::Device,
    label: &'static str,
    size: UVec2,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}
