use zircon_math::UVec2;

use super::clear_texture::clear_texture;
use super::scene_frame_history_textures::SceneFrameHistoryTextures;
use super::texture_extent::texture_extent;

impl SceneFrameHistoryTextures {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: UVec2) -> Self {
        let scene_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-scene-color"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::core::OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let scene_color_view = scene_color.create_view(&wgpu::TextureViewDescriptor::default());
        let ambient_occlusion = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-ambient-occlusion"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let ambient_occlusion_view =
            ambient_occlusion.create_view(&wgpu::TextureViewDescriptor::default());

        clear_texture(queue, &scene_color, size, &[0, 0, 0, 255]);
        clear_texture(queue, &ambient_occlusion, size, &[255, 255, 255, 255]);

        Self {
            size,
            scene_color,
            scene_color_view,
            ambient_occlusion,
            ambient_occlusion_view,
        }
    }
}
