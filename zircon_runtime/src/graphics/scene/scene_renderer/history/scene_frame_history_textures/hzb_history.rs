use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::hzb::{HzbBuilder, HzbSampledResourceIdentity};

use super::super::texture_extent::texture_extent;

pub(super) struct HzbHistoryTexture {
    identity: HzbSampledResourceIdentity,
    size: UVec2,
    mip_count: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl HzbHistoryTexture {
    pub(super) fn new(device: &wgpu::Device, render_size: UVec2) -> Self {
        let plan = HzbBuilder::new(render_size).build_plan();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-hzb-furthest"),
            size: texture_extent(plan.hzb_size),
            mip_level_count: plan.mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            identity: HzbSampledResourceIdentity::new(),
            size: plan.hzb_size,
            mip_count: plan.mip_count,
            texture,
            view,
        }
    }

    pub(super) const fn identity(&self) -> HzbSampledResourceIdentity {
        self.identity
    }

    pub(super) const fn size(&self) -> UVec2 {
        self.size
    }

    pub(super) const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub(super) const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(super) const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
