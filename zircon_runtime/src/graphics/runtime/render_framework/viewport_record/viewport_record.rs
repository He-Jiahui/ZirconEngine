use crate::core::framework::render::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderViewportDescriptor,
};

use crate::{
    runtime::{HybridGiRuntimeState, ViewportFrameHistory, VirtualGeometryRuntimeState},
    CompiledRenderPipeline,
};

pub(in crate::graphics::runtime::render_framework) struct ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) descriptor: RenderViewportDescriptor,
    pub(in crate::graphics::runtime::render_framework) pipeline: Option<RenderPipelineHandle>,
    pub(in crate::graphics::runtime::render_framework) quality_profile:
        Option<RenderQualityProfile>,
    pub(in crate::graphics::runtime::render_framework) compiled_pipeline:
        Option<CompiledRenderPipeline>,
    pub(in crate::graphics::runtime::render_framework) last_capture: Option<CapturedFrame>,
    pub(in crate::graphics::runtime::render_framework) history: Option<ViewportFrameHistory>,
    pub(in crate::graphics::runtime::render_framework) hybrid_gi_runtime:
        Option<HybridGiRuntimeState>,
    pub(in crate::graphics::runtime::render_framework) virtual_geometry_runtime:
        Option<VirtualGeometryRuntimeState>,
}
