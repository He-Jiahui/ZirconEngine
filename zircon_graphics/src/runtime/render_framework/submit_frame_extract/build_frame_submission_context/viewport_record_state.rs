use zircon_framework::render::RenderPipelineHandle;
use zircon_math::UVec2;

use crate::{
    runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState},
    RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityHistorySnapshot,
};

pub(super) struct ViewportRecordState {
    pub(super) size: UVec2,
    pub(super) pipeline_handle: RenderPipelineHandle,
    pub(super) quality_profile: Option<String>,
    pub(super) previous_visibility: Option<VisibilityHistorySnapshot>,
    pub(super) previous_hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    pub(super) previous_virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    pub(super) pipeline_asset: RenderPipelineAsset,
    pub(super) compile_options: RenderPipelineCompileOptions,
    pub(super) predicted_generation: u64,
}
