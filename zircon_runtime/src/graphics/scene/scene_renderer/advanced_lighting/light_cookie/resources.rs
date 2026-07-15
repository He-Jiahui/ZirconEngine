use std::sync::Arc;

use crate::core::framework::render::LightCookieData;
use crate::graphics::scene::resources::ResourceStreamer;

use super::blit_pipeline::LightCookieAtlasBlitPipeline;
use super::{build_cookie_frame_plan, COOKIE_ATLAS_GRID_SIZE};

pub(crate) const LIGHT_COOKIE_ATLAS_BINDING: u32 = 33;
pub(crate) const LIGHT_COOKIE_SAMPLER_BINDING: u32 = 34;
pub(crate) const LIGHT_COOKIE_ATLAS_SIZE: u32 = 1024;

pub(crate) struct LightCookieAtlasResources {
    texture: wgpu::Texture,
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>,
    blit_pipeline: LightCookieAtlasBlitPipeline,
}

impl LightCookieAtlasResources {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        queue.write_texture(
            texture.as_image_copy(),
            &vec![255; (LIGHT_COOKIE_ATLAS_SIZE * LIGHT_COOKIE_ATLAS_SIZE * 4) as usize],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(LIGHT_COOKIE_ATLAS_SIZE * 4),
                rows_per_image: Some(LIGHT_COOKIE_ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: LIGHT_COOKIE_ATLAS_SIZE,
                height: LIGHT_COOKIE_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );
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
        }
    }

    pub(crate) fn rebuild(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        cookies: &[LightCookieData],
    ) -> usize {
        let _retain_texture = &self.texture;
        let plan = build_cookie_frame_plan(cookies);
        self.blit_pipeline.encode(
            device,
            encoder,
            &self.view,
            streamer,
            plan.entries(),
            LIGHT_COOKIE_ATLAS_SIZE / COOKIE_ATLAS_GRID_SIZE,
        )
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
