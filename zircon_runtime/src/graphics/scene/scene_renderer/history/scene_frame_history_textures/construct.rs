use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::params::exposure_params::default_exposure_buffer_words;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TemporalHistoryStore, TAA_SCENE_COLOR_HISTORY_FORMAT,
};
use crate::graphics::visibility::HzbBuilder;
use wgpu::util::DeviceExt;

use super::super::clear_texture::clear_texture;
use super::super::texture_extent::texture_extent;
use super::scene_frame_history_textures::SceneFrameHistoryTextures;

const RGBA16_FLOAT_BYTES_PER_TEXEL: u32 = 8;
const RGBA16_FLOAT_BLACK_CONFIDENCE_ZERO: [u8; RGBA16_FLOAT_BYTES_PER_TEXEL as usize] =
    [0, 0, 0, 0, 0, 0, 0, 0];

impl SceneFrameHistoryTextures {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: UVec2,
        render_size: UVec2,
    ) -> Self {
        let hzb_plan = HzbBuilder::new(render_size).build_plan();
        let taa_scene_color_read = create_scene_color_history_texture(
            device,
            crate::core::framework::render::PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
            size,
            TAA_SCENE_COLOR_HISTORY_FORMAT,
        );
        let taa_scene_color_read_view =
            taa_scene_color_read.create_view(&wgpu::TextureViewDescriptor::default());
        let taa_scene_color_write = create_scene_color_history_texture(
            device,
            crate::core::framework::render::PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            size,
            TAA_SCENE_COLOR_HISTORY_FORMAT,
        );
        let taa_scene_color_write_view =
            taa_scene_color_write.create_view(&wgpu::TextureViewDescriptor::default());
        clear_rgba16_float_texture(queue, &taa_scene_color_read, size);
        clear_rgba16_float_texture(queue, &taa_scene_color_write, size);
        let taa_scene_color = TemporalHistoryStore::new(
            TemporalHistoryKey::new(size, TAA_SCENE_COLOR_HISTORY_FORMAT),
            taa_scene_color_read,
            taa_scene_color_read_view,
            taa_scene_color_write,
            taa_scene_color_write_view,
        );
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
        let exposure_read = create_exposure_history_buffer(device, "zircon-history-exposure-read");
        let exposure_write =
            create_exposure_history_buffer(device, "zircon-history-exposure-write");

        clear_texture(queue, &global_illumination, size, &[0, 0, 0, 255]);
        clear_texture(queue, &ambient_occlusion, size, &[255, 255, 255, 255]);
        clear_texture(queue, &screen_space_reflection, size, &[0, 0, 0, 0]);

        Self {
            size,
            hzb_furthest_size: hzb_plan.hzb_size,
            hzb_furthest_mip_count: hzb_plan.mip_count,
            taa_scene_color,
            global_illumination,
            global_illumination_view,
            ambient_occlusion,
            ambient_occlusion_view,
            screen_space_reflection,
            screen_space_reflection_view,
            hzb_furthest,
            hzb_furthest_view,
            exposure_read,
            exposure_write,
        }
    }
}

fn create_exposure_history_buffer(device: &wgpu::Device, label: &'static str) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&default_exposure_buffer_words()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_scene_color_history_texture(
    device: &wgpu::Device,
    label: &'static str,
    size: UVec2,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn clear_rgba16_float_texture(queue: &wgpu::Queue, texture: &wgpu::Texture, size: UVec2) {
    let texel_count = size.x.max(1) as usize * size.y.max(1) as usize;
    let mut data = Vec::with_capacity(texel_count * RGBA16_FLOAT_BYTES_PER_TEXEL as usize);
    for _ in 0..texel_count {
        data.extend_from_slice(&RGBA16_FLOAT_BLACK_CONFIDENCE_ZERO);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.x.max(1) * RGBA16_FLOAT_BYTES_PER_TEXEL),
            rows_per_image: Some(size.y.max(1)),
        },
        texture_extent(size),
    );
}
