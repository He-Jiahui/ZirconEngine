use std::sync::Arc;

use crate::core::framework::render::LightCookieData;
use crate::graphics::scene::resources::ResourceStreamer;

use super::blit_pipeline::LightCookieAtlasBlitPipeline;
use super::profile::LightCookieAtlasProfile;
use super::{build_cookie_frame_plan, COOKIE_ATLAS_GRID_SIZE};

pub(crate) const LIGHT_COOKIE_ATLAS_BINDING: u32 = 33;
pub(crate) const LIGHT_COOKIE_SAMPLER_BINDING: u32 = 34;
pub(crate) const LIGHT_COOKIE_ATLAS_SIZE: u32 = 1024;

/// 网格管线缓存持有的固定图集与 blit 管线，跨帧复用。
/// 重建时独立生成槽位计划，需与光缓冲保持相同帧输入与去重规则。
pub(crate) struct LightCookieAtlasResources {
    texture: wgpu::Texture,
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>,
    blit_pipeline: LightCookieAtlasBlitPipeline,
    profile: LightCookieAtlasProfile,
}

impl LightCookieAtlasResources {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        crate::profile_scope!("render", "light_cookie", "atlas_construct");
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-light-cookie-atlas"),
            size: wgpu::Extent3d {
                width: LIGHT_COOKIE_ATLAS_SIZE,
                height: LIGHT_COOKIE_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-light-cookie-atlas-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        }));
        Self {
            texture,
            view,
            sampler,
            blit_pipeline: LightCookieAtlasBlitPipeline::new(device),
            profile: LightCookieAtlasProfile::default(),
        }
    }

    pub(crate) fn rebuild(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        cookies: &[LightCookieData],
    ) -> usize {
        let _retain_texture = &self.texture;
        let plan = {
            crate::profile_scope!("render", "light_cookie", "frame_plan");
            build_cookie_frame_plan(cookies)
        };
        let resolved_draw_count = {
            crate::profile_scope!("render", "light_cookie", "atlas_encode");
            self.blit_pipeline.encode(
                device,
                encoder,
                &self.view,
                streamer,
                plan.entries(),
                LIGHT_COOKIE_ATLAS_SIZE / COOKIE_ATLAS_GRID_SIZE,
            )
        };
        self.profile.record_rebuild(
            cookies.len(),
            plan.entries().len(),
            resolved_draw_count,
            u64::from(LIGHT_COOKIE_ATLAS_SIZE) * u64::from(LIGHT_COOKIE_ATLAS_SIZE),
        );
        resolved_draw_count
    }

    pub(crate) fn begin_profile_frame(&mut self) {
        self.profile.begin_frame();
    }

    pub(crate) fn emit_profile_frame(&self) {
        self.profile.emit();
    }

    pub(crate) fn bind_group_entries(&self) -> [wgpu::BindGroupEntry<'_>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: LIGHT_COOKIE_ATLAS_BINDING,
                resource: wgpu::BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: LIGHT_COOKIE_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

pub(crate) fn light_cookie_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; 2] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_COOKIE_ATLAS_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_COOKIE_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

#[cfg(test)]
#[path = "tests/resources.rs"]
mod tests;
