use crate::core::framework::render::{RenderCapabilitySummary, RenderPipelineHandle};
use crate::core::math::UVec2;

use crate::{
    runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState},
    RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityHistorySnapshot,
};

pub(super) struct ViewportRecordState {
    size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    quality_profile: Option<String>,
    previous_visibility: Option<VisibilityHistorySnapshot>,
    previous_hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    previous_virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    pipeline_asset: RenderPipelineAsset,
    compile_options: RenderPipelineCompileOptions,
    capabilities: RenderCapabilitySummary,
    predicted_generation: u64,
}

impl ViewportRecordState {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        size: UVec2,
        pipeline_handle: RenderPipelineHandle,
        quality_profile: Option<String>,
        previous_visibility: Option<VisibilityHistorySnapshot>,
        previous_hybrid_gi_runtime: Option<HybridGiRuntimeState>,
        previous_virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
        pipeline_asset: RenderPipelineAsset,
        compile_options: RenderPipelineCompileOptions,
        capabilities: RenderCapabilitySummary,
        predicted_generation: u64,
    ) -> Self {
        Self {
            size,
            pipeline_handle,
            quality_profile,
            previous_visibility,
            previous_hybrid_gi_runtime,
            previous_virtual_geometry_runtime,
            pipeline_asset,
            compile_options,
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

    pub(super) fn previous_visibility(&self) -> Option<&VisibilityHistorySnapshot> {
        self.previous_visibility.as_ref()
    }

    pub(super) fn pipeline_asset(&self) -> &RenderPipelineAsset {
        &self.pipeline_asset
    }

    pub(super) fn compile_options(&self) -> &RenderPipelineCompileOptions {
        &self.compile_options
    }

    pub(super) fn capabilities(&self) -> &RenderCapabilitySummary {
        &self.capabilities
    }

    pub(super) fn take_quality_profile(&mut self) -> Option<String> {
        self.quality_profile.take()
    }

    pub(super) fn take_previous_hybrid_gi_runtime(&mut self) -> Option<HybridGiRuntimeState> {
        self.previous_hybrid_gi_runtime.take()
    }

    pub(super) fn take_previous_virtual_geometry_runtime(
        &mut self,
    ) -> Option<VirtualGeometryRuntimeState> {
        self.previous_virtual_geometry_runtime.take()
    }

    pub(super) fn predicted_generation(&self) -> u64 {
        self.predicted_generation
    }
}
