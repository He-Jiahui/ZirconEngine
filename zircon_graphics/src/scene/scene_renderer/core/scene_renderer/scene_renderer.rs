use std::collections::HashMap;
use std::sync::Arc;

use zircon_render_server::FrameHistoryHandle;

use super::super::scene_renderer_core::SceneRendererCore;
use crate::backend::{OffscreenTarget, RenderBackend};
use crate::scene::resources::ResourceStreamer;
use crate::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::scene::scene_renderer::HybridGiGpuReadback;
use crate::scene::scene_renderer::VirtualGeometryGpuReadback;

pub struct SceneRenderer {
    pub(in crate::scene::scene_renderer::core) backend: RenderBackend,
    pub(in crate::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::scene::scene_renderer::core) generation: u64,
    pub(in crate::scene::scene_renderer::core) last_hybrid_gi_gpu_readback:
        Option<HybridGiGpuReadback>,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_gpu_readback:
        Option<VirtualGeometryGpuReadback>,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_indirect_draw_count: u32,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_indirect_buffer_count: u32,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_indirect_segment_count: u32,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::scene::scene_renderer::core) last_virtual_geometry_indirect_args_count: u32,
}
