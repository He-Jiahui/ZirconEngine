use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::core::math::UVec2;
use crate::graphics::visibility::HzbBuilder;

use super::super::clear_texture::clear_texture;
use super::super::texture_extent::texture_extent;
use super::scene_frame_history_textures::SceneFrameHistoryTextures;

impl SceneFrameHistoryTextures {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: UVec2,
        render_size: UVec2,
    ) -> Self {
        let hzb_plan = HzbBuilder::new(render_size).build_plan();
        let scene_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(PostProcessGraphResourceNames::HISTORY_CURRENT_SCENE_COLOR),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::super::core::OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let scene_color_view = scene_color.create_view(&wgpu::TextureViewDescriptor::default());
        let global_illumination = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-global-illumination"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::super::core::OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let global_illumination_view =
            global_illumination.create_view(&wgpu::TextureViewDescriptor::default());
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
        let screen_space_reflection = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-screen-space-reflection"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::super::core::OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let screen_space_reflection_view =
            screen_space_reflection.create_view(&wgpu::TextureViewDescriptor::default());
        let hzb_furthest = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-hzb-furthest"),
            size: texture_extent(hzb_plan.hzb_size),
            mip_level_count: hzb_plan.mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let hzb_furthest_view = hzb_furthest.create_view(&wgpu::TextureViewDescriptor::default());

        clear_texture(queue, &scene_color, size, &[0, 0, 0, 255]);
        clear_texture(queue, &global_illumination, size, &[0, 0, 0, 255]);
        clear_texture(queue, &ambient_occlusion, size, &[255, 255, 255, 255]);
        clear_texture(queue, &screen_space_reflection, size, &[0, 0, 0, 0]);

        Self {
            size,
            hzb_furthest_size: hzb_plan.hzb_size,
            hzb_furthest_mip_count: hzb_plan.mip_count,
            scene_color,
            scene_color_view,
            global_illumination,
            global_illumination_view,
            ambient_occlusion,
            ambient_occlusion_view,
            screen_space_reflection,
            screen_space_reflection_view,
            hzb_furthest,
            hzb_furthest_view,
        }
    }
}
