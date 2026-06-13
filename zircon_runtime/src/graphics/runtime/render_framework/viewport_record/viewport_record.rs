use crate::core::framework::render::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderViewportDescriptor,
    ViewportCameraSnapshot,
};
use crate::graphics::ViewportMotionVectorObjectHistory;

use crate::graphics::visibility::VisibilityStaticIndex;
use crate::{
    graphics::backend::ViewportSurface, runtime::ViewportFrameHistory, CompiledRenderPipeline,
    HybridGiRuntimeState, VirtualGeometryRuntimeState,
};

pub(in crate::graphics::runtime::render_framework) struct ViewportRecord {
    pub(super) descriptor: RenderViewportDescriptor,
    pub(super) pipeline: Option<RenderPipelineHandle>,
    pub(super) quality_profile: Option<RenderQualityProfile>,
    pub(super) generation: u64,
    pub(super) compiled_pipeline: Option<CompiledRenderPipeline>,
    pub(super) hybrid_gi_runtime: Option<Box<dyn HybridGiRuntimeState>>,
    pub(super) virtual_geometry_runtime: Option<Box<dyn VirtualGeometryRuntimeState>>,
    pub(super) last_capture: Option<CapturedFrame>,
    pub(super) history: Option<ViewportFrameHistory>,
    pub(super) visibility_static_index: Option<VisibilityStaticIndex>,
    pub(super) motion_vector_camera: Option<ViewportCameraSnapshot>,
    pub(super) motion_vector_object_history: Option<ViewportMotionVectorObjectHistory>,
    pub(super) surface: Option<ViewportSurface>,
}
