use crate::asset::{RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage,
};
use std::sync::Arc;

use super::super::{GpuTextureResource, TextureSamplerCache};

pub(in crate::graphics::scene::resources) fn create_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    sampler_cache: Arc<TextureSamplerCache>,
) -> GpuTextureResource {
    create_solid_fallback_texture(
        device,
        queue,
        texture_layout,
        sampler_cache,
        "zircon-fallback-texture",
        "zircon-fallback-bind-group",
        [255, 255, 255, 255],
        RGBA8_UNORM_SRGB_FORMAT,
        RenderImageColorSpace::Srgb,
        RenderImageFallbackKind::OpaqueWhite,
    )
}

pub(in crate::graphics::scene::resources) fn create_fallback_normal_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    sampler_cache: Arc<TextureSamplerCache>,
) -> GpuTextureResource {
    create_solid_fallback_texture(
        device,
        queue,
        texture_layout,
        sampler_cache,
        "zircon-fallback-normal-texture",
        "zircon-fallback-normal-bind-group",
        [128, 128, 255, 255],
        RGBA8_UNORM_FORMAT,
        RenderImageColorSpace::Linear,
        RenderImageFallbackKind::NormalMap,
    )
}

fn create_solid_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    sampler_cache: Arc<TextureSamplerCache>,
    texture_label: &'static str,
    bind_group_label: &'static str,
    rgba: [u8; 4],
    format: &str,
    color_space: RenderImageColorSpace,
    fallback: RenderImageFallbackKind,
) -> GpuTextureResource {
    let descriptor = RenderImageDescriptor {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
        dimension: RenderImageDimension::D2,
        format: format.to_string(),
        color_space,
        metadata: crate::core::framework::render::TextureMetadata {
            color_space,
            ..Default::default()
        },
        sampler: crate::core::framework::render::RenderSamplerDescriptor::default(),
        usage: vec![RenderImageUsage::Sampled],
        asset_usage: Vec::new(),
        mip_count: 1,
        array_layer_count: 1,
        fallback,
    };
    let texture_format = match format {
        RGBA8_UNORM_FORMAT => wgpu::TextureFormat::Rgba8Unorm,
        RGBA8_UNORM_SRGB_FORMAT => wgpu::TextureFormat::Rgba8UnormSrgb,
        unsupported => panic!("unsupported fallback texture format `{unsupported}`"),
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(texture_label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = sampler_cache.sampler_for_image(device, &descriptor);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(bind_group_label),
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
            },
        ],
    });
    GpuTextureResource {
        id: None,
        descriptor,
        texture,
        view,
        sampler,
        sampler_cache,
        mip_streaming_supported: false,
        resident_texture_bytes: 4,
        bind_group,
    }
}
