use std::collections::HashMap;

use zircon_render_server::FrameHistoryHandle;

use crate::backend::{OffscreenTarget, RenderBackend};
use crate::scene::scene_renderer::HybridGiGpuReadback;
use crate::scene::scene_renderer::VirtualGeometryGpuReadback;

use super::super::super::resources::ResourceStreamer;
use super::super::history::SceneFrameHistoryTextures;
use super::scene_renderer_core::SceneRendererCore;

pub struct SceneRenderer {
    pub(super) backend: RenderBackend,
    pub(super) core: SceneRendererCore,
    pub(super) streamer: ResourceStreamer,
    pub(super) target: Option<OffscreenTarget>,
    pub(super) history_targets: HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(super) generation: u64,
    pub(super) last_hybrid_gi_gpu_readback: Option<HybridGiGpuReadback>,
    pub(super) last_virtual_geometry_gpu_readback: Option<VirtualGeometryGpuReadback>,
}
