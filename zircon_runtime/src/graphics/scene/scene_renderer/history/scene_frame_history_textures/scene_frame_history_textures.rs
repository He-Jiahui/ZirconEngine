use crate::core::framework::render::{FroxelGridQuality, RenderHistoryDomainsReport};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TemporalHistoryStore,
};
use crate::rhi::{BufferDesc, TextureDesc, TextureDimension, TextureFormat, TextureUsage};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::{
    ExposureHistoryBuffers, GlobalIlluminationHistory, HzbHistoryTexture,
    SceneFrameHistoryRequirements, SceneHistoryDomain, SceneHistoryDomainStates,
    SceneHistoryFrameTransaction, ScreenSpaceReflectionHistory, VolumetricHistoryTexture,
};

pub(crate) struct SceneFrameHistoryTextures {
    pub(crate) size: UVec2,
    pub(crate) render_size: UVec2,
    pub(super) requirements: SceneFrameHistoryRequirements,
    pub(super) taa_scene_color: Option<TemporalHistoryStore>,
    pub(super) global_illumination: Option<GlobalIlluminationHistory>,
    pub(super) volumetric_scattering: Option<VolumetricHistoryTexture>,
    pub(super) screen_space_reflection: Option<ScreenSpaceReflectionHistory>,
    pub(super) hzb_furthest: Option<HzbHistoryTexture>,
    pub(super) exposure: Option<ExposureHistoryBuffers>,
    pub(super) domain_states: SceneHistoryDomainStates,
}

impl SceneFrameHistoryTextures {
    pub(crate) fn hzb_resource_identity(&self) -> Option<HzbSampledResourceIdentity> {
        self.hzb_furthest.as_ref().map(HzbHistoryTexture::identity)
    }

    pub(crate) fn hzb_furthest_texture(&self) -> Option<&wgpu::Texture> {
        self.hzb_furthest.as_ref().map(HzbHistoryTexture::texture)
    }

    pub(crate) fn hzb_furthest_view(&self) -> Option<&wgpu::TextureView> {
        self.hzb_furthest.as_ref().map(HzbHistoryTexture::view)
    }

    pub(crate) fn hzb_furthest_size(&self) -> Option<UVec2> {
        self.hzb_furthest.as_ref().map(HzbHistoryTexture::size)
    }

    pub(crate) fn hzb_furthest_mip_count(&self) -> Option<u32> {
        self.hzb_furthest.as_ref().map(HzbHistoryTexture::mip_count)
    }

    pub(crate) fn hzb_furthest_desc(&self, label: &'static str) -> Option<TextureDesc> {
        let history = self.hzb_furthest.as_ref()?;
        Some(
            TextureDesc::new(
                label,
                history.size().x,
                history.size().y,
                TextureFormat::Rgba16Float,
                TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            )
            .with_mip_levels(history.mip_count()),
        )
    }

    pub(crate) fn volumetric_history_quality(&self) -> Option<FroxelGridQuality> {
        self.volumetric_scattering
            .as_ref()
            .map(|history| history.quality)
    }

    pub(crate) fn volumetric_history_view(&self) -> Option<&wgpu::TextureView> {
        self.volumetric_scattering
            .as_ref()
            .map(|history| &history.view)
    }

    pub(crate) fn volumetric_history_texture(&self) -> Option<&wgpu::Texture> {
        self.volumetric_scattering
            .as_ref()
            .map(|history| &history.texture)
    }

    pub(crate) fn volumetric_history_desc(&self, label: &'static str) -> Option<TextureDesc> {
        let [width, height, depth] = self.volumetric_history_quality()?.dimensions();
        Some(
            TextureDesc::new(
                label,
                width,
                height,
                TextureFormat::Rgba16Float,
                TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            )
            .with_dimension(TextureDimension::D3)
            .with_depth(depth),
        )
    }

    pub(crate) fn taa_scene_color_history_matches(&self, key: TemporalHistoryKey) -> bool {
        self.taa_scene_color
            .as_ref()
            .is_some_and(|history| history.matches_key(key))
    }

    pub(crate) fn taa_scene_color_previous_view(&self) -> Option<&wgpu::TextureView> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::previous_view)
    }

    pub(crate) fn taa_scene_color_previous_texture(&self) -> Option<&wgpu::Texture> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::previous_texture)
    }

    pub(crate) fn taa_scene_color_previous_identity(
        &self,
    ) -> Option<crate::graphics::resource_identity::SampledTextureIdentity> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::previous_identity)
    }

    pub(crate) fn taa_scene_color_current_view(&self) -> Option<&wgpu::TextureView> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::current_view)
    }

    pub(crate) fn taa_scene_color_current_texture(&self) -> Option<&wgpu::Texture> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::current_texture)
    }

    pub(crate) fn taa_scene_color_current_identity(
        &self,
    ) -> Option<crate::graphics::resource_identity::SampledTextureIdentity> {
        self.taa_scene_color
            .as_ref()
            .map(TemporalHistoryStore::current_identity)
    }

    pub(crate) fn taa_scene_color_desc(&self, label: &'static str) -> Option<TextureDesc> {
        self.taa_scene_color.as_ref()?;
        Some(TextureDesc::new(
            label,
            self.size.x,
            self.size.y,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_DST,
        ))
    }

    pub(crate) fn global_illumination_texture(&self) -> Option<&wgpu::Texture> {
        self.global_illumination
            .as_ref()
            .map(GlobalIlluminationHistory::lighting)
    }

    pub(crate) fn global_illumination_view(&self) -> Option<&wgpu::TextureView> {
        self.global_illumination
            .as_ref()
            .map(GlobalIlluminationHistory::lighting_view)
    }

    pub(crate) fn global_illumination_temporal_metadata_texture(&self) -> Option<&wgpu::Texture> {
        self.global_illumination
            .as_ref()
            .map(GlobalIlluminationHistory::temporal_metadata)
    }

    pub(crate) fn global_illumination_temporal_metadata_view(&self) -> Option<&wgpu::TextureView> {
        self.global_illumination
            .as_ref()
            .map(GlobalIlluminationHistory::temporal_metadata_view)
    }

    pub(crate) fn global_illumination_desc(&self, label: &'static str) -> Option<TextureDesc> {
        self.global_illumination.as_ref()?;
        Some(TextureDesc::new(
            label,
            self.size.x,
            self.size.y,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_DST,
        ))
    }

    pub(crate) fn screen_space_reflection_texture(&self) -> Option<&wgpu::Texture> {
        self.screen_space_reflection
            .as_ref()
            .map(ScreenSpaceReflectionHistory::texture)
    }

    pub(crate) fn screen_space_reflection_view(&self) -> Option<&wgpu::TextureView> {
        self.screen_space_reflection
            .as_ref()
            .map(ScreenSpaceReflectionHistory::view)
    }

    pub(crate) fn screen_space_reflection_desc(&self, label: &'static str) -> Option<TextureDesc> {
        self.screen_space_reflection.as_ref()?;
        Some(TextureDesc::new(
            label,
            self.size.x,
            self.size.y,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_DST,
        ))
    }

    pub(crate) fn request_exposure_history_reset(&mut self) {
        if let Some(exposure) = self.exposure.as_mut() {
            exposure.request_reset();
        }
    }

    pub(crate) fn prepare_exposure_history_reset(
        &self,
        uploads: &mut WgpuBufferUploadBatch,
    ) -> bool {
        self.exposure
            .as_ref()
            .is_some_and(|exposure| exposure.prepare_reset(uploads))
    }

    pub(crate) fn commit_exposure_history_reset(&mut self) -> bool {
        self.exposure
            .as_mut()
            .is_some_and(ExposureHistoryBuffers::commit_reset)
    }

    pub(crate) fn exposure_previous_buffer(&self) -> Option<&wgpu::Buffer> {
        self.exposure.as_ref().map(ExposureHistoryBuffers::read)
    }

    pub(crate) fn exposure_current_buffer(&self) -> Option<&wgpu::Buffer> {
        self.exposure.as_ref().map(ExposureHistoryBuffers::write)
    }

    pub(crate) fn exposure_buffer_desc(&self, label: &'static str) -> Option<BufferDesc> {
        self.exposure.as_ref().map(|exposure| exposure.desc(label))
    }

    pub(crate) fn begin_history_frame(&self) -> SceneHistoryFrameTransaction {
        SceneHistoryFrameTransaction::begin(&self.domain_states)
    }

    pub(crate) fn commit_history_frame(
        &mut self,
        transaction: SceneHistoryFrameTransaction,
        frame_generation: u64,
    ) -> RenderHistoryDomainsReport {
        if transaction.domain_was_written(SceneHistoryDomain::TaaSceneColor) {
            if let Some(taa_scene_color) = self.taa_scene_color.as_mut() {
                taa_scene_color.flip_after_success();
            }
        }
        if transaction.domain_was_written(SceneHistoryDomain::Exposure) {
            if let Some(exposure) = self.exposure.as_mut() {
                exposure.flip_after_success();
            }
        }
        transaction.commit(&mut self.domain_states, frame_generation)
    }

    #[cfg(test)]
    fn exposure_history_reset_pending(&self) -> bool {
        self.exposure
            .as_ref()
            .is_some_and(ExposureHistoryBuffers::reset_pending)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::UVec2;
    use crate::graphics::backend::RenderBackend;
    use zr_rhi_wgpu::WgpuBufferUploadBatch;

    use super::{SceneFrameHistoryRequirements, SceneFrameHistoryTextures};

    #[test]
    fn exposure_history_reset_commits_only_after_prepared_upload_acceptance() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let requirements =
            SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
        let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
            &backend.device,
            UVec2::splat(8),
            UVec2::splat(8),
            requirements,
        );
        assert!(!history.exposure_history_reset_pending());

        history.request_exposure_history_reset();
        let mut uploads = WgpuBufferUploadBatch::new();
        assert!(history.prepare_exposure_history_reset(&mut uploads));
        assert!(!uploads.is_empty());
        assert!(history.exposure_history_reset_pending());

        assert!(history.commit_exposure_history_reset());
        assert!(!history.exposure_history_reset_pending());
    }

    #[test]
    fn exposure_only_requirements_create_no_image_history_or_clear_commands() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let requirements =
            SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
        let (history, initialization_command_buffer) =
            SceneFrameHistoryTextures::new_with_requirements_and_initialization(
                &backend.device,
                UVec2::new(32, 16),
                UVec2::new(16, 8),
                requirements,
            );

        assert!(initialization_command_buffer.is_none());
        assert!(history.exposure_previous_buffer().is_some());
        assert!(history.taa_scene_color_previous_texture().is_none());
        assert!(history.global_illumination_texture().is_none());
        assert!(history.screen_space_reflection_texture().is_none());
        assert!(history.hzb_furthest_texture().is_none());
        assert!(history.volumetric_history_texture().is_none());
    }

    #[test]
    fn enabling_ssr_preserves_unrelated_taa_and_hzb_allocations() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let initial = SceneFrameHistoryRequirements::new(true, false, false, true, false, None);
        let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
            &backend.device,
            UVec2::new(32, 16),
            UVec2::new(16, 8),
            initial,
        );
        let taa_identity = history.taa_scene_color_current_identity().unwrap();
        let hzb_identity = history.hzb_resource_identity().unwrap();
        let with_ssr = SceneFrameHistoryRequirements::new(true, false, true, true, false, None);

        let (changes, initialization_command_buffer) = history
            .reconcile_with_requirements_and_initialization(
                &backend.device,
                UVec2::new(32, 16),
                UVec2::new(16, 8),
                with_ssr,
            );

        assert!(changes.changed(super::SceneHistoryDomain::ScreenSpaceReflection));
        assert!(!changes.changed(super::SceneHistoryDomain::TaaSceneColor));
        assert!(!changes.changed(super::SceneHistoryDomain::HzbFurthest));
        assert!(initialization_command_buffer.is_some());
        assert_eq!(
            history.taa_scene_color_current_identity(),
            Some(taa_identity)
        );
        assert_eq!(history.hzb_resource_identity(), Some(hzb_identity));
        assert!(history.screen_space_reflection_texture().is_some());
    }

    #[test]
    fn disabling_image_history_releases_only_that_domain_without_clear_commands() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let initial = SceneFrameHistoryRequirements::new(false, false, true, false, true, None);
        let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
            &backend.device,
            UVec2::new(32, 16),
            UVec2::new(16, 8),
            initial,
        );
        let exposure_size = history.exposure_previous_buffer().unwrap().size();
        let exposure_only =
            SceneFrameHistoryRequirements::new(false, false, false, false, true, None);

        let (changes, initialization_command_buffer) = history
            .reconcile_with_requirements_and_initialization(
                &backend.device,
                UVec2::new(32, 16),
                UVec2::new(16, 8),
                exposure_only,
            );

        assert!(changes.changed(super::SceneHistoryDomain::ScreenSpaceReflection));
        assert!(!changes.changed(super::SceneHistoryDomain::Exposure));
        assert!(initialization_command_buffer.is_none());
        assert!(history.screen_space_reflection_texture().is_none());
        assert_eq!(
            history.exposure_previous_buffer().unwrap().size(),
            exposure_size
        );
    }

    #[test]
    fn shared_scene_history_does_not_allocate_unqualified_ambient_occlusion_storage() {
        let owner = include_str!("scene_frame_history_textures.rs");
        let owner = owner.split("#[cfg(test)]").next().unwrap();
        let construct = include_str!("construct.rs");
        let construct = construct.split("#[cfg(test)]").next().unwrap();

        assert!(!owner.contains("ambient_occlusion: wgpu::Texture"));
        assert!(!owner.contains("ambient_occlusion_texture("));
        assert!(!owner.contains("ambient_occlusion_view("));
        assert!(!owner.contains("ambient_occlusion_desc("));
        assert!(!construct.contains("zircon-history-ambient-occlusion"));
    }

    #[test]
    fn exposure_history_reset_uses_frame_upload_transaction_without_raw_queue() {
        let source = include_str!("scene_frame_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let render =
            include_str!("../../core/scene_renderer_core_render_compiled_scene/render/render.rs");
        let submit = include_str!(
            "../../core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs"
        );

        assert!(production.contains("prepare_exposure_history_reset"));
        assert!(!production.contains("queue.write_buffer("));
        assert!(!production.contains("queue: &wgpu::Queue"));
        let accept = render
            .find(".enqueue_copy_resource_upload_batch(")
            .expect("frame owner must accept the merged upload batch");
        let ledger = render[accept..]
            .find("RenderFrameSubmissionProducer::FrameResourceUpload")
            .map(|offset| accept + offset)
            .expect("frame upload ticket must enter the ledger");
        let scene_submit = submit
            .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
            .expect("compiled scene must reach its ticketed submit boundary");
        let commit = submit
            .find("commit_exposure_history_reset")
            .expect("accepted exposure reset intent must commit after scene submission");
        assert!(accept < ledger);
        assert!(scene_submit < commit);
    }
}
