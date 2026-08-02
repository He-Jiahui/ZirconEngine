use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    pub(super) compiled_pipeline: Option<Arc<CompiledRenderPipeline>>,
    pub(super) last_capture_pipeline: Option<Arc<CompiledRenderPipeline>>,
    pub(super) capture_mailbox: Arc<Mutex<ViewportAsyncCaptureMailbox>>,
    pub(super) pending_capture_profiles:
        std::collections::BTreeMap<u64, Vec<crate::core::framework::render::RenderFrameProfile>>,
    pub(super) last_promoted_capture_generation: Option<u64>,
    pub(super) hybrid_gi_runtimes: HashMap<ViewportCameraHistoryKey, Box<dyn HybridGiRuntimeState>>,
    pub(super) virtual_geometry_runtimes:
        HashMap<ViewportCameraHistoryKey, Box<dyn VirtualGeometryRuntimeState>>,
    pub(super) light_grid_reports:
        HashMap<ViewportCameraHistoryKey, crate::graphics::scene::RenderGraphLightGridReport>,
    pub(super) virtual_geometry_debug_snapshots:
        HashMap<ViewportCameraHistoryKey, Arc<RenderVirtualGeometryDebugSnapshot>>,
    pub(super) last_capture: Option<CapturedFrame>,
    pub(super) last_visible_spatial_query:
        Option<Arc<crate::core::framework::render::RenderVisibleSpatialQuerySnapshot>>,
    pub(super) camera_histories: HashMap<ViewportCameraHistoryKey, ViewportFrameHistory>,
    pub(super) motion_vector_cameras: HashMap<ViewportCameraHistoryKey, ViewportCameraSnapshot>,
    pub(super) particle_previous_sprites:
        HashMap<ViewportCameraHistoryKey, Vec<RenderParticlePreviousSpriteSnapshot>>,
    pub(super) surface: Option<ViewportSurface>,
}

pub(super) struct ViewportAsyncCaptureMailbox {
    pub(super) pending: std::collections::BTreeMap<u64, PendingViewportCapture>,
    pub(super) completed: std::collections::BTreeMap<u64, Result<Vec<u8>, String>>,
    pub(super) ready: Option<ReadyViewportCapture>,
}

pub(super) struct PendingViewportCapture {
    pub(super) size: crate::core::math::UVec2,
    pub(super) capture_report: crate::core::framework::render::RenderCaptureReport,
    pub(super) pipeline: Arc<CompiledRenderPipeline>,
}

pub(super) struct ReadyViewportCapture {
    pub(super) capture: CapturedFrame,
    pub(super) pipeline: Arc<CompiledRenderPipeline>,
}
