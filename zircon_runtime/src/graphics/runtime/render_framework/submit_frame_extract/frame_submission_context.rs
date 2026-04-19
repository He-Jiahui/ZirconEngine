use crate::core::framework::render::{
    RenderHybridGiExtract, RenderPipelineHandle, RenderVirtualGeometryExtract,
};
use crate::core::math::UVec2;

use crate::{
    runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState},
    CompiledRenderPipeline, VisibilityContext, VisibilityHybridGiFeedback,
    VisibilityHybridGiUpdatePlan, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct UiSubmissionStats {
    pub(super) command_count: usize,
    pub(super) quad_count: usize,
    pub(super) text_payload_count: usize,
    pub(super) image_payload_count: usize,
    pub(super) clipped_command_count: usize,
}

pub(super) struct FrameSubmissionContext {
    pub(super) size: UVec2,
    pub(super) pipeline_handle: RenderPipelineHandle,
    pub(super) quality_profile: Option<String>,
    pub(super) compiled_pipeline: CompiledRenderPipeline,
    pub(super) visibility_context: VisibilityContext,
    pub(super) ui_stats: UiSubmissionStats,
    pub(super) previous_hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    pub(super) previous_virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    pub(super) hybrid_gi_enabled: bool,
    pub(super) virtual_geometry_enabled: bool,
    pub(super) hybrid_gi_extract: Option<RenderHybridGiExtract>,
    pub(super) hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
    pub(super) hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
    pub(super) virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
    pub(super) virtual_geometry_page_upload_plan: Option<VisibilityVirtualGeometryPageUploadPlan>,
    pub(super) virtual_geometry_feedback: Option<VisibilityVirtualGeometryFeedback>,
    pub(super) predicted_generation: u64,
}
