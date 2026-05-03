use crate::core::framework::render::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderViewportDescriptor,
};

use crate::{
    runtime::ViewportFrameHistory, CompiledRenderPipeline, HybridGiRuntimeState,
    VirtualGeometryRuntimeState,
};

pub(in crate::graphics::runtime::render_framework) struct ViewportRecord {
    pub(super) descriptor: RenderViewportDescriptor,
    pub(super) pipeline: Option<RenderPipelineHandle>,
    pub(super) quality_profile: Option<RenderQualityProfile>,
    pub(super) compiled_pipeline: Option<CompiledRenderPipeline>,
    pub(super) hybrid_gi_runtime: Option<Box<dyn HybridGiRuntimeState>>,
    pub(super) virtual_geometry_runtime: Option<Box<dyn VirtualGeometryRuntimeState>>,
    pub(super) last_capture: Option<CapturedFrame>,
    pub(super) history: Option<ViewportFrameHistory>,
}
