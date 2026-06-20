use std::collections::HashMap;

use crate::core::framework::render::{
    CapturedFrame, RenderParticlePreviousSpriteSnapshot, RenderPipelineHandle,
    RenderQualityProfile, RenderViewportDescriptor, RenderVirtualGeometryDebugSnapshot,
    ViewportCameraSnapshot,
};

use crate::graphics::{
    backend::ViewportSurface, runtime::ViewportFrameHistory, CompiledRenderPipeline,
    HybridGiRuntimeState, VirtualGeometryRuntimeState,
};

use super::ViewportCameraHistoryKey;

pub(in crate::graphics::runtime::render_framework) struct ViewportRecord {
    pub(super) descriptor: RenderViewportDescriptor,
    pub(super) pipeline: Option<RenderPipelineHandle>,
    pub(super) quality_profile: Option<RenderQualityProfile>,
    pub(super) generation: u64,
    pub(super) temporal_frame_index: u64,
    pub(super) compiled_pipeline: Option<CompiledRenderPipeline>,
    pub(super) hybrid_gi_runtimes: HashMap<ViewportCameraHistoryKey, Box<dyn HybridGiRuntimeState>>,
    pub(super) virtual_geometry_runtimes:
        HashMap<ViewportCameraHistoryKey, Box<dyn VirtualGeometryRuntimeState>>,
    pub(super) light_grid_reports:
        HashMap<ViewportCameraHistoryKey, crate::graphics::scene::RenderGraphLightGridReport>,
    pub(super) virtual_geometry_debug_snapshots:
        HashMap<ViewportCameraHistoryKey, RenderVirtualGeometryDebugSnapshot>,
    pub(super) last_capture: Option<CapturedFrame>,
    pub(super) camera_histories: HashMap<ViewportCameraHistoryKey, ViewportFrameHistory>,
    pub(super) motion_vector_cameras: HashMap<ViewportCameraHistoryKey, ViewportCameraSnapshot>,
    pub(super) particle_previous_sprites:
        HashMap<ViewportCameraHistoryKey, Vec<RenderParticlePreviousSpriteSnapshot>>,
    pub(super) surface: Option<ViewportSurface>,
}
