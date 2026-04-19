use crate::core::math::UVec2;

use crate::graphics::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};

use super::super::offscreen_target::OffscreenTarget;
use super::create_cluster_buffer::create_cluster_buffer;
use super::create_texture_bundle::create_texture_bundle;

impl OffscreenTarget {
    pub(crate) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        let final_color = create_texture_bundle(
            device,
            "zircon-offscreen-final-color",
            size,
            OFFSCREEN_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let scene_color = create_texture_bundle(
            device,
            "zircon-offscreen-scene-color",
            size,
            OFFSCREEN_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let bloom = create_texture_bundle(
            device,
            "zircon-offscreen-bloom",
            size,
            OFFSCREEN_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let gbuffer_albedo = create_texture_bundle(
            device,
            "zircon-offscreen-gbuffer-albedo",
            size,
            GBUFFER_ALBEDO_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let normal = create_texture_bundle(
            device,
            "zircon-offscreen-normal",
            size,
            NORMAL_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let ambient_occlusion = create_texture_bundle(
            device,
            "zircon-offscreen-ambient-occlusion",
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = create_depth_texture(device, size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let cluster_dimensions = cluster_dimensions_for_size(size);
        let cluster_buffer_bytes = cluster_buffer_bytes_for_size(size);
        let cluster_buffer = create_cluster_buffer(device, cluster_buffer_bytes);

        Self {
            size,
            final_color: final_color.texture,
            final_color_view: final_color.view,
            scene_color: scene_color.texture,
            scene_color_view: scene_color.view,
            bloom: bloom.texture,
            bloom_view: bloom.view,
            gbuffer_albedo: gbuffer_albedo.texture,
            gbuffer_albedo_view: gbuffer_albedo.view,
            normal: normal.texture,
            normal_view: normal.view,
            ambient_occlusion: ambient_occlusion.texture,
            ambient_occlusion_view: ambient_occlusion.view,
            depth,
            depth_view,
            cluster_dimensions,
            cluster_buffer,
            cluster_buffer_bytes,
        }
    }
}
