use zircon_math::UVec2;

use crate::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};

use super::offscreen_target::OffscreenTarget;

impl OffscreenTarget {
    pub(crate) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        let final_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-final-color"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let final_color_view = final_color.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-scene-color"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let scene_color_view = scene_color.create_view(&wgpu::TextureViewDescriptor::default());
        let bloom = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-bloom"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let bloom_view = bloom.create_view(&wgpu::TextureViewDescriptor::default());
        let gbuffer_albedo = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-gbuffer-albedo"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: GBUFFER_ALBEDO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gbuffer_albedo_view =
            gbuffer_albedo.create_view(&wgpu::TextureViewDescriptor::default());
        let normal = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-normal"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: NORMAL_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let normal_view = normal.create_view(&wgpu::TextureViewDescriptor::default());
        let ambient_occlusion = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-ambient-occlusion"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let ambient_occlusion_view =
            ambient_occlusion.create_view(&wgpu::TextureViewDescriptor::default());
        let depth = create_depth_texture(device, size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let cluster_dimensions = cluster_dimensions_for_size(size);
        let cluster_buffer_bytes = cluster_buffer_bytes_for_size(size);
        let cluster_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-cluster-buffer"),
            size: cluster_buffer_bytes.max(16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            size,
            final_color,
            final_color_view,
            scene_color,
            scene_color_view,
            bloom,
            bloom_view,
            gbuffer_albedo,
            gbuffer_albedo_view,
            normal,
            normal_view,
            ambient_occlusion,
            ambient_occlusion_view,
            depth,
            depth_view,
            cluster_dimensions,
            cluster_buffer,
            cluster_buffer_bytes,
        }
    }
}
