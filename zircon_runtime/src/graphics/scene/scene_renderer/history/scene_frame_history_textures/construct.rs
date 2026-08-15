use crate::core::framework::render::FroxelGridQuality;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::params::exposure_params::default_exposure_buffer_words;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TemporalHistoryStore, TAA_SCENE_COLOR_HISTORY_FORMAT,
};
use crate::graphics::visibility::HzbBuilder;
use wgpu::util::DeviceExt;

use super::super::texture_extent::texture_extent;
use super::scene_frame_history_textures::SceneFrameHistoryTextures;
use super::VolumetricHistoryTexture;

impl SceneFrameHistoryTextures {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: UVec2,
        render_size: UVec2,
    ) -> Self {
        Self::new_with_volumetric_history(device, queue, size, render_size, None)
    }

    pub(crate) fn new_with_volumetric_history(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: UVec2,
        render_size: UVec2,
        volumetric_quality: Option<FroxelGridQuality>,
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
        let global_illumination = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-global-illumination"),
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
        let global_illumination_view =
            global_illumination.create_view(&wgpu::TextureViewDescriptor::default());
        let global_illumination_temporal_metadata =
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("zircon-history-global-illumination-temporal-metadata"),
                size: texture_extent(size),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        let global_illumination_temporal_metadata_view = global_illumination_temporal_metadata
            .create_view(&wgpu::TextureViewDescriptor::default());
        let ambient_occlusion = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-history-ambient-occlusion"),
            size: texture_extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
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
            format: super::super::super::core::SCENE_COLOR_HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
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

        clear_history_textures(
            device,
            queue,
            &taa_scene_color_read_view,
            &taa_scene_color_write_view,
            &global_illumination_view,
            &global_illumination_temporal_metadata_view,
            &ambient_occlusion_view,
            &screen_space_reflection_view,
        );
        let taa_scene_color = TemporalHistoryStore::new(
            TemporalHistoryKey::new(size, TAA_SCENE_COLOR_HISTORY_FORMAT),
            taa_scene_color_read,
            taa_scene_color_read_view,
            taa_scene_color_write,
            taa_scene_color_write_view,
        );

        Self {
            hzb_resource_identity:
                crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity::new(),
            size,
            hzb_furthest_size: hzb_plan.hzb_size,
            hzb_furthest_mip_count: hzb_plan.mip_count,
            taa_scene_color,
            global_illumination,
            global_illumination_view,
            global_illumination_temporal_metadata,
            global_illumination_temporal_metadata_view,
            global_illumination_history_valid: false,
            volumetric_scattering: volumetric_quality
                .map(|quality| VolumetricHistoryTexture::new(device, quality)),
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

#[allow(clippy::too_many_arguments)]
fn clear_history_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    taa_read_view: &wgpu::TextureView,
    taa_write_view: &wgpu::TextureView,
    global_illumination_view: &wgpu::TextureView,
    global_illumination_metadata_view: &wgpu::TextureView,
    ambient_occlusion_view: &wgpu::TextureView,
    screen_space_reflection_view: &wgpu::TextureView,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-history-initialize-encoder"),
    });
    {
        let color_attachments = [
            clear_attachment(taa_read_view, wgpu::Color::TRANSPARENT),
            clear_attachment(taa_write_view, wgpu::Color::TRANSPARENT),
            clear_attachment(global_illumination_view, wgpu::Color::TRANSPARENT),
            clear_attachment(global_illumination_metadata_view, wgpu::Color::TRANSPARENT),
        ];
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-history-initialize-hdr-pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }
    {
        let color_attachments = [
            clear_attachment(screen_space_reflection_view, wgpu::Color::TRANSPARENT),
            clear_attachment(
                ambient_occlusion_view,
                wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            ),
        ];
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-history-initialize-ssr-ao-pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }
    queue.submit([encoder.finish()]);
}

fn clear_attachment(
    view: &wgpu::TextureView,
    color: wgpu::Color,
) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(color),
            store: wgpu::StoreOp::Store,
        },
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_initialization_uses_gpu_clear_passes_without_cpu_texture_payloads() {
        let source = include_str!("construct.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("history texture construction implementation");

        assert!(!implementation.contains("Vec::with_capacity("));
        assert!(!implementation.contains("write_texture("));
        assert_eq!(implementation.matches("begin_render_pass(").count(), 2);
        assert!(implementation.contains("zircon-history-initialize-hdr-pass"));
        assert!(implementation.contains("zircon-history-initialize-ssr-ao-pass"));
    }
}
