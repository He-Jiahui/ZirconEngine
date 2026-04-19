use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::FrameHistoryHandle;

use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::backend::{OffscreenTarget, RenderBackend};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::HybridGiGpuReadback;
use crate::graphics::scene::scene_renderer::VirtualGeometryGpuReadback;

pub struct SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) backend: RenderBackend,
    pub(in crate::graphics::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::graphics::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::graphics::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::graphics::scene::scene_renderer::core) generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) last_hybrid_gi_gpu_readback:
        Option<HybridGiGpuReadback>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_gpu_readback:
        Option<VirtualGeometryGpuReadback>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_buffer_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_order:
        Vec<(u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_records:
        Vec<(u64, u32, u64, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_args_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_draw_refs_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_segments_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_execution_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_execution_records_buffer:
        Option<Arc<wgpu::Buffer>>,
}
