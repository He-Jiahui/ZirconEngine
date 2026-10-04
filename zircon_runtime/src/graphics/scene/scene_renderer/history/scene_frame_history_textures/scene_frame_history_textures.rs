use crate::core::framework::render::{FroxelGridQuality, RenderHistoryDomainsReport};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TemporalHistoryStore,
};
use crate::rhi::{BufferDesc, TextureDesc, TextureDimension, TextureFormat, TextureUsage};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::{SceneHistoryDomain, SceneHistoryDomainStates, SceneHistoryFrameTransaction};
use super::{
    ExposureHistoryBuffers, GlobalIlluminationHistory, HzbHistoryTexture,
    SceneFrameHistoryRequirements, ScreenSpaceReflectionHistory, VolumetricHistoryTexture,
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

    // 调用者须先获得场景提交 ticket；只有事务的成功写入位才翻转 TAA/exposure 读写端，
    // 其余域由事务报告独立更新，不在这里等待 GPU 完成。
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
#[path = "tests/scene_frame_history_textures.rs"]
mod tests;
