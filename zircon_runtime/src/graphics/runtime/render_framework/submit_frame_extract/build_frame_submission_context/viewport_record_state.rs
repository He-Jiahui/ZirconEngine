use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, RenderCapabilitySummary, RenderPipelineHandle, SolariRuntimeReport,
};
use crate::core::math::UVec2;

use crate::{RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityHistorySnapshot};

pub(super) struct ViewportRecordState {
    size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    viewport_generation: u64,
    quality_profile: Option<String>,
    previous_visibility: Option<VisibilityHistorySnapshot>,
    pipeline_asset: RenderPipelineAsset,
    compile_options: RenderPipelineCompileOptions,
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
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
        quality_profile: Option<String>,
        previous_visibility: Option<VisibilityHistorySnapshot>,
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
            quality_profile,
            previous_visibility,
            pipeline_asset,
            compile_options,
            advanced_runtime_plan,
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

    pub(super) fn previous_visibility(&self) -> Option<&VisibilityHistorySnapshot> {
        self.previous_visibility.as_ref()
    }

    pub(super) fn pipeline_asset(&self) -> &RenderPipelineAsset {
        &self.pipeline_asset
    }

    pub(super) fn compile_options(&self) -> &RenderPipelineCompileOptions {
        &self.compile_options
    }

    pub(super) fn advanced_runtime_plan(&self) -> &AdvancedProfileRuntimePlan {
        &self.advanced_runtime_plan
    }

    pub(super) fn solari_runtime_report(&self) -> &SolariRuntimeReport {
        &self.solari_runtime_report
    }

    pub(super) fn capabilities(&self) -> &RenderCapabilitySummary {
        &self.capabilities
    }

    pub(super) fn take_quality_profile(&mut self) -> Option<String> {
        self.quality_profile.take()
    }

    pub(super) fn predicted_generation(&self) -> u64 {
        self.predicted_generation
    }
}
