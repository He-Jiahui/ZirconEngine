use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TAA_SCENE_COLOR_HISTORY_FORMAT, TemporalHistoryKey, TemporalHistoryStore,
};

use super::super::texture_extent::texture_extent;
use super::super::{
    ExposureHistoryBuffers, GlobalIlluminationHistory, HzbHistoryTexture,
    SceneFrameHistoryRequirements, SceneHistoryAllocationChanges, SceneHistoryDomain,
    ScreenSpaceReflectionHistory,
};
use super::VolumetricHistoryTexture;
use super::scene_frame_history_textures::SceneFrameHistoryTextures;

impl SceneFrameHistoryTextures {
    pub(crate) fn new_with_requirements_and_initialization(
        device: &wgpu::Device,
        size: UVec2,
        render_size: UVec2,
        requirements: SceneFrameHistoryRequirements,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        let taa_scene_color = requirements
            .taa_scene_color()
            .then(|| create_taa_scene_color_history(device, size));
        let global_illumination = requirements
            .hybrid_global_illumination()
            .then(|| GlobalIlluminationHistory::new(device, size));
        let screen_space_reflection = requirements
            .screen_space_reflection()
            .then(|| ScreenSpaceReflectionHistory::new(device, size));
        let hzb_furthest = requirements
            .hzb_furthest()
            .then(|| HzbHistoryTexture::new(device, render_size));
        let exposure = requirements
            .exposure()
            .then(|| ExposureHistoryBuffers::new(device));
        let volumetric_scattering = requirements
            .volumetric_scattering()
            .map(|quality| VolumetricHistoryTexture::new(device, quality));

        let initialization_command_buffer = clear_history_textures(
            device,
            taa_scene_color.as_ref(),
            global_illumination.as_ref(),
            screen_space_reflection.as_ref(),
        );

        (
            Self {
                size,
                render_size,
                requirements,
                taa_scene_color,
                global_illumination,
                volumetric_scattering,
                screen_space_reflection,
                hzb_furthest,
                exposure,
                domain_states: Default::default(),
            },
            initialization_command_buffer,
        )
    }

    pub(crate) fn reconcile_with_requirements_and_initialization(
        &mut self,
        device: &wgpu::Device,
        size: UVec2,
        render_size: UVec2,
        requirements: SceneFrameHistoryRequirements,
    ) -> (SceneHistoryAllocationChanges, Option<wgpu::CommandBuffer>) {
        let changes = self.requirements.allocation_changes(
            self.size,
            self.render_size,
            requirements,
            size,
            render_size,
        );
        if changes.is_empty() {
            self.size = size;
            self.render_size = render_size;
            self.requirements = requirements;
            return (changes, None);
        }

        let taa_scene_color = changes.changed(SceneHistoryDomain::TaaSceneColor).then(|| {
            requirements
                .taa_scene_color()
                .then(|| create_taa_scene_color_history(device, size))
        });
        let global_illumination = changes
            .changed(SceneHistoryDomain::HybridGlobalIllumination)
            .then(|| {
                requirements
                    .hybrid_global_illumination()
                    .then(|| GlobalIlluminationHistory::new(device, size))
            });
        let screen_space_reflection = changes
            .changed(SceneHistoryDomain::ScreenSpaceReflection)
            .then(|| {
                requirements
                    .screen_space_reflection()
                    .then(|| ScreenSpaceReflectionHistory::new(device, size))
            });
        let hzb_furthest = changes.changed(SceneHistoryDomain::HzbFurthest).then(|| {
            requirements
                .hzb_furthest()
                .then(|| HzbHistoryTexture::new(device, render_size))
        });
        let exposure = changes.changed(SceneHistoryDomain::Exposure).then(|| {
            requirements
                .exposure()
                .then(|| ExposureHistoryBuffers::new(device))
        });
        let volumetric_scattering = changes
            .changed(SceneHistoryDomain::VolumetricScattering)
            .then(|| {
                requirements
                    .volumetric_scattering()
                    .map(|quality| VolumetricHistoryTexture::new(device, quality))
            });

        let initialization_command_buffer = clear_history_textures(
            device,
            taa_scene_color.as_ref().and_then(Option::as_ref),
            global_illumination.as_ref().and_then(Option::as_ref),
            screen_space_reflection.as_ref().and_then(Option::as_ref),
        );

        if let Some(replacement) = taa_scene_color {
            self.taa_scene_color = replacement;
        }
        if let Some(replacement) = global_illumination {
            self.global_illumination = replacement;
        }
        if let Some(replacement) = screen_space_reflection {
            self.screen_space_reflection = replacement;
        }
        if let Some(replacement) = hzb_furthest {
            self.hzb_furthest = replacement;
        }
        if let Some(replacement) = exposure {
            self.exposure = replacement;
        }
        if let Some(replacement) = volumetric_scattering {
            self.volumetric_scattering = replacement;
        }
        self.size = size;
        self.render_size = render_size;
        self.requirements = requirements;

        (changes, initialization_command_buffer)
    }
}

fn create_taa_scene_color_history(device: &wgpu::Device, size: UVec2) -> TemporalHistoryStore {
    let read = create_scene_color_history_texture(
        device,
        crate::core::framework::render::PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
        size,
        TAA_SCENE_COLOR_HISTORY_FORMAT,
    );
    let read_view = read.create_view(&wgpu::TextureViewDescriptor::default());
    let write = create_scene_color_history_texture(
        device,
        crate::core::framework::render::PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
        size,
        TAA_SCENE_COLOR_HISTORY_FORMAT,
    );
    let write_view = write.create_view(&wgpu::TextureViewDescriptor::default());
    TemporalHistoryStore::new(
        TemporalHistoryKey::new(size, TAA_SCENE_COLOR_HISTORY_FORMAT),
        read,
        read_view,
        write,
        write_view,
    )
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

fn clear_history_textures(
    device: &wgpu::Device,
    taa_scene_color: Option<&TemporalHistoryStore>,
    global_illumination: Option<&GlobalIlluminationHistory>,
    screen_space_reflection: Option<&ScreenSpaceReflectionHistory>,
) -> Option<wgpu::CommandBuffer> {
    if taa_scene_color.is_none()
        && global_illumination.is_none()
        && screen_space_reflection.is_none()
    {
        return None;
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-history-initialize-encoder"),
    });
    let hdr_attachments = [
        taa_scene_color
            .map(|history| clear_attachment(history.previous_view(), wgpu::Color::TRANSPARENT)),
        taa_scene_color
            .map(|history| clear_attachment(history.current_view(), wgpu::Color::TRANSPARENT)),
        global_illumination
            .map(|history| clear_attachment(history.lighting_view(), wgpu::Color::TRANSPARENT)),
        global_illumination.map(|history| {
            clear_attachment(history.temporal_metadata_view(), wgpu::Color::TRANSPARENT)
        }),
    ];
    let mut encoded_clear = false;
    if hdr_attachments.iter().any(Option::is_some) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-history-initialize-hdr-pass"),
            color_attachments: &hdr_attachments,
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        encoded_clear = true;
    }
    if let Some(screen_space_reflection) = screen_space_reflection {
        let color_attachments = [Some(clear_attachment(
            screen_space_reflection.view(),
            wgpu::Color::TRANSPARENT,
        ))];
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-history-initialize-ssr-pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        encoded_clear = true;
    }
    debug_assert!(encoded_clear);
    Some(encoder.finish())
}

fn clear_attachment(
    view: &wgpu::TextureView,
    color: wgpu::Color,
) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(color),
            store: wgpu::StoreOp::Store,
        },
    }
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
        assert!(implementation.contains("zircon-history-initialize-ssr-pass"));
        assert!(!implementation.contains("zircon-history-ambient-occlusion"));
        assert!(
            implementation.contains("let initialization_command_buffer = clear_history_textures(")
        );
        assert!(implementation.contains("initialization_command_buffer,"));
        assert!(implementation.contains("screen_space_reflection.is_none()"));
        assert!(implementation.contains("return None"));
        assert!(implementation.contains("Some(encoder.finish())"));
        assert!(!implementation.contains("submit_graphics_command_buffers("));
        assert!(!implementation.contains("enqueue_graphics_command_buffers("));
        assert!(!implementation.contains("queue.submit("));
    }

    #[test]
    fn every_physical_history_owner_is_guarded_by_compiled_requirements() {
        let source = include_str!("construct.rs");
        let implementation = source.split("#[cfg(test)]").next().unwrap();
        let constructor = implementation
            .split("pub(crate) fn new_with_requirements_and_initialization(")
            .nth(1)
            .and_then(|tail| {
                tail.split("pub(crate) fn reconcile_with_requirements_and_initialization(")
                    .next()
            })
            .expect("initial history constructor");

        for requirement in [
            ".taa_scene_color()",
            ".hybrid_global_illumination()",
            ".screen_space_reflection()",
            ".hzb_furthest()",
            ".exposure()",
            ".volumetric_scattering()",
        ] {
            assert!(implementation.contains(requirement));
        }
        assert_eq!(constructor.matches(".then(||").count(), 5);
        assert!(constructor.contains(".map(|quality| VolumetricHistoryTexture::new("));
    }

    #[test]
    fn reconcile_applies_replacements_only_after_clear_commands_are_encoded() {
        let source = include_str!("construct.rs");
        let implementation = source.split("#[cfg(test)]").next().unwrap();
        let clear = implementation
            .rfind("let initialization_command_buffer = clear_history_textures(")
            .expect("reconcile must encode clears for newly-created attachments");
        let first_assignment = implementation[clear..]
            .find("self.taa_scene_color = replacement;")
            .map(|offset| clear + offset)
            .expect("reconcile must publish the TAA replacement");
        let commit_requirements = implementation[clear..]
            .find("self.requirements = requirements;")
            .map(|offset| clear + offset)
            .expect("reconcile must publish requirements after clear encoding");

        assert!(clear < first_assignment);
        assert!(first_assignment < commit_requirements);
    }
}
