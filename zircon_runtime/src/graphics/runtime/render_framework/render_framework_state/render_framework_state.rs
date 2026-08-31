use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RenderPipelineHandle, RenderStats, RenderViewportHandle, RenderVirtualGeometryDebugSnapshot,
};
use crate::graphics::pipeline::CompiledGraphCache;
use crate::graphics::scene::EnvironmentCaptureSubmission;

use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderPipelineAsset, SceneRenderer,
    SolariRuntimeProviderRegistration, VirtualGeometryRuntimeProviderRegistration,
};

use super::super::budget::{BudgetDegradeLadder, GpuMemoryBudget};
use super::super::frame_profiler::FrameProfiler;
use super::super::graphics_debugger_capture::GraphicsDebuggerState;
use super::super::viewport_record::ViewportRecord;
use super::{EnvironmentCaptureResidency, EnvironmentIblHydrationCache};

pub(in crate::graphics::runtime::render_framework) struct RenderFrameworkState {
    pub(in crate::graphics::runtime::render_framework) renderer: SceneRenderer,
    pub(in crate::graphics::runtime::render_framework) pending_environment_capture_submission:
        Option<EnvironmentCaptureSubmission>,
    pub(in crate::graphics::runtime::render_framework) environment_capture_residency:
        EnvironmentCaptureResidency,
    // The renderer retains one offscreen scene-color target. Track its source so
    // HDR capture cannot return pixels produced for a different viewport.
    pub(in crate::graphics::runtime::render_framework) last_retained_scene_color_viewport:
        Option<RenderViewportHandle>,
    pub(in crate::graphics::runtime::render_framework) next_viewport_id: u64,
    pub(in crate::graphics::runtime::render_framework) next_history_id: u64,
    pub(in crate::graphics::runtime::render_framework) pipelines:
        HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pub(in crate::graphics::runtime::render_framework) compiled_graph_cache: CompiledGraphCache,
    pub(in crate::graphics::runtime::render_framework) environment_ibl_hydration_cache:
        Arc<Mutex<EnvironmentIblHydrationCache>>,
    pub(in crate::graphics::runtime::render_framework) hybrid_gi_runtime_provider:
        Option<HybridGiRuntimeProviderRegistration>,
    pub(in crate::graphics::runtime::render_framework) solari_runtime_provider:
        Option<SolariRuntimeProviderRegistration>,
    pub(in crate::graphics::runtime::render_framework) virtual_geometry_runtime_provider:
        Option<VirtualGeometryRuntimeProviderRegistration>,
    pub(in crate::graphics::runtime::render_framework) last_virtual_geometry_debug_snapshot:
        Option<Arc<RenderVirtualGeometryDebugSnapshot>>,
    pub(in crate::graphics::runtime::render_framework) viewports:
        HashMap<RenderViewportHandle, ViewportRecord>,
    pub(in crate::graphics::runtime::render_framework) stats: RenderStats,
    pub(in crate::graphics::runtime::render_framework) frame_profiler: FrameProfiler,
    pub(in crate::graphics::runtime::render_framework) memory_budget: GpuMemoryBudget,
    pub(in crate::graphics::runtime::render_framework) degrade_ladder: BudgetDegradeLadder,
    pub(in crate::graphics::runtime::render_framework) graphics_debugger: GraphicsDebuggerState,
    pub(in crate::graphics::runtime::render_framework) viewport_products:
        Arc<super::ViewportProductRegistry>,
    pub(in crate::graphics::runtime::render_framework) viewport_pick_frames:
        super::ViewportPickFrameRegistry,
    pub(in crate::graphics::runtime::render_framework) viewport_picks: super::ViewportPickStore,
}
