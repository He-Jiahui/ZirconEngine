use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::FrameHistoryHandle;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

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
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_debug_snapshot:
        Option<RenderVirtualGeometryDebugSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_draw_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_buffer_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_page_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_resident_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_pending_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_missing_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_repeated_draw_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_execution_indirect_offsets:
        Vec<u64>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_order:
        Vec<(Option<u32>, u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_records:
        Vec<(u64, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_mesh_draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_args_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_draw_refs_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_segments_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_execution_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_indirect_execution_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_cull_input_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_instance_seed_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_node_and_cluster_cull_instance_seed_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_selected_cluster_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_visbuffer64_clear_value:
        u64,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_visbuffer64_source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_visbuffer64_entry_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_visbuffer64_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_hardware_rasterization_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::graphics::scene::scene_renderer::core) last_virtual_geometry_hardware_rasterization_buffer:
        Option<Arc<wgpu::Buffer>>,
}
