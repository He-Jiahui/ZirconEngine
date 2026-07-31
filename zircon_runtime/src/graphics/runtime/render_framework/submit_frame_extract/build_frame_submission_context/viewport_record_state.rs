use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, RenderCapabilitySummary, RenderParticlePreviousSpriteSnapshot,
    RenderPipelineHandle, ShaderQualityTier, SolariRuntimeReport, TaaQualityPreset,
    ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::visibility::VisibilityStaticIndex;

use crate::graphics::{
    RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityHistorySnapshot,
};

pub(super) struct ViewportRecordState {
    size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    viewport_generation: u64,
    temporal_frame_index: u64,
    quality_profile: Option<String>,
    shader_quality: ShaderQualityTier,
    quality_profile_taa_quality: Option<TaaQualityPreset>,
    previous_visibility: Option<VisibilityHistorySnapshot>,
    previous_static_index: Option<VisibilityStaticIndex>,
    previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
    previous_particle_sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
    pipeline_asset: RenderPipelineAsset,
    compile_options: RenderPipelineCompileOptions,
    advanced_runtime_plan: Option<AdvancedProfileRuntimePlan>,
    solari_runtime_report: SolariRuntimeReport,
    capabilities: RenderCapabilitySummary,
    predicted_generation: u64,
}

impl ViewportRecordState {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        size: UVec2,
        pipeline_handle: RenderPipelineHandle,
        viewport_generation: u64,
        temporal_frame_index: u64,
        quality_profile: Option<String>,
        shader_quality: ShaderQualityTier,
        quality_profile_taa_quality: Option<TaaQualityPreset>,
        previous_visibility: Option<VisibilityHistorySnapshot>,
        previous_static_index: Option<VisibilityStaticIndex>,
        previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
        previous_particle_sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
        pipeline_asset: RenderPipelineAsset,
        compile_options: RenderPipelineCompileOptions,
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
        solari_runtime_report: SolariRuntimeReport,
        capabilities: RenderCapabilitySummary,
        predicted_generation: u64,
    ) -> Self {
        Self {
            size,
            pipeline_handle,
            viewport_generation,
            temporal_frame_index,
            quality_profile,
            shader_quality,
            quality_profile_taa_quality,
            previous_visibility,
            previous_static_index,
            previous_motion_vector_camera,
            previous_particle_sprites,
            pipeline_asset,
            compile_options,
            advanced_runtime_plan: Some(advanced_runtime_plan),
            solari_runtime_report,
            capabilities,
            predicted_generation,
        }
    }

    pub(super) fn size(&self) -> UVec2 {
        self.size
    }

    pub(super) fn pipeline_handle(&self) -> RenderPipelineHandle {
        self.pipeline_handle
    }

    pub(super) fn viewport_generation(&self) -> u64 {
        self.viewport_generation
    }

    pub(super) fn temporal_frame_index(&self) -> u64 {
        self.temporal_frame_index
    }

    pub(super) fn previous_visibility(&self) -> Option<&VisibilityHistorySnapshot> {
        self.previous_visibility.as_ref()
    }

    pub(super) fn previous_static_index(&self) -> Option<&VisibilityStaticIndex> {
        self.previous_static_index.as_ref()
    }

    pub(super) fn take_previous_motion_vector_camera(&mut self) -> Option<ViewportCameraSnapshot> {
        self.previous_motion_vector_camera.take()
    }

    pub(super) fn take_previous_particle_sprites(
        &mut self,
    ) -> Vec<RenderParticlePreviousSpriteSnapshot> {
        std::mem::take(&mut self.previous_particle_sprites)
    }

    pub(super) fn pipeline_asset(&self) -> &RenderPipelineAsset {
        &self.pipeline_asset
    }

    pub(super) fn compile_options(&self) -> &RenderPipelineCompileOptions {
        &self.compile_options
    }

    pub(super) fn take_advanced_runtime_plan(&mut self) -> AdvancedProfileRuntimePlan {
        self.advanced_runtime_plan
            .take()
            .expect("advanced runtime plan is moved into one submission context")
    }

    pub(super) fn take_solari_runtime_report(&mut self) -> SolariRuntimeReport {
        std::mem::take(&mut self.solari_runtime_report)
    }

    pub(super) fn capabilities(&self) -> &RenderCapabilitySummary {
        &self.capabilities
    }

    pub(super) fn take_capabilities(&mut self) -> RenderCapabilitySummary {
        std::mem::take(&mut self.capabilities)
    }

    pub(super) fn take_quality_profile(&mut self) -> Option<String> {
        self.quality_profile.take()
    }

    pub(super) fn shader_quality(&self) -> ShaderQualityTier {
        self.shader_quality
    }

    pub(super) fn quality_profile_taa_quality(&self) -> Option<TaaQualityPreset> {
        self.quality_profile_taa_quality
    }

    pub(super) fn predicted_generation(&self) -> u64 {
        self.predicted_generation
    }
}
