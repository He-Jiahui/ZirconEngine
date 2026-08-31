use crate::core::framework::render::{FroxelGridQuality, PostProcessGraphResourceNames};

pub(super) struct VolumetricHistoryTexture {
    pub(super) quality: FroxelGridQuality,
    pub(super) texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
}

impl VolumetricHistoryTexture {
    pub(super) fn new(device: &wgpu::Device, quality: FroxelGridQuality) -> Self {
        let [width, height, depth] = quality.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            quality,
            texture,
            view,
        }
    }
}
