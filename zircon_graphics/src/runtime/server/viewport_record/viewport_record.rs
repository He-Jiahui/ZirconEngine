use zircon_render_server::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderViewportDescriptor,
};

use crate::{
    runtime::{HybridGiRuntimeState, ViewportFrameHistory, VirtualGeometryRuntimeState},
    CompiledRenderPipeline,
};

pub(in crate::runtime::server) struct ViewportRecord {
    pub(in crate::runtime::server) descriptor: RenderViewportDescriptor,
    pub(in crate::runtime::server) pipeline: Option<RenderPipelineHandle>,
    pub(in crate::runtime::server) quality_profile: Option<RenderQualityProfile>,
    pub(in crate::runtime::server) compiled_pipeline: Option<CompiledRenderPipeline>,
    pub(in crate::runtime::server) last_capture: Option<CapturedFrame>,
    pub(in crate::runtime::server) history: Option<ViewportFrameHistory>,
    pub(in crate::runtime::server) hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    pub(in crate::runtime::server) virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
}
