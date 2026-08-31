use crate::core::math::UVec2;

use super::super::texture_extent::texture_extent;

pub(super) struct ScreenSpaceReflectionHistory {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl ScreenSpaceReflectionHistory {
    pub(super) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-screen-space-reflection"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::super::core::SCENE_COLOR_HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }

    pub(super) const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(super) const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
