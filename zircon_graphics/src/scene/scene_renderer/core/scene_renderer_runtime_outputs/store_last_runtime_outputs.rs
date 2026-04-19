use std::sync::Arc;

use crate::scene::scene_renderer::{HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback};
use crate::types::GraphicsError;

use super::super::scene_renderer::SceneRenderer;

#[allow(clippy::too_many_arguments)]
pub(in crate::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    hybrid_gi_gpu_readback: Option<HybridGiGpuPendingReadback>,
    virtual_geometry_gpu_readback: Option<VirtualGeometryGpuPendingReadback>,
    indirect_draw_count: u32,
    indirect_buffer_count: u32,
    indirect_segment_count: u32,
    indirect_draw_submission_order: Vec<(u64, u32)>,
    indirect_draw_submission_records: Vec<(u64, u32, u64, usize)>,
    indirect_draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_count: u32,
    indirect_submission_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> Result<(), GraphicsError> {
    renderer.last_hybrid_gi_gpu_readback = hybrid_gi_gpu_readback
        .map(|pending| pending.collect(&renderer.backend.device))
        .transpose()?;
    renderer.last_virtual_geometry_gpu_readback = virtual_geometry_gpu_readback
        .map(|pending| pending.collect(&renderer.backend.device))
        .transpose()?;
    renderer.last_virtual_geometry_indirect_draw_count = indirect_draw_count;
    renderer.last_virtual_geometry_indirect_buffer_count = indirect_buffer_count;
    renderer.last_virtual_geometry_indirect_segment_count = indirect_segment_count;
    renderer.last_virtual_geometry_mesh_draw_submission_order = indirect_draw_submission_order;
    renderer.last_virtual_geometry_mesh_draw_submission_records = indirect_draw_submission_records;
    renderer.last_virtual_geometry_mesh_draw_submission_token_records =
        indirect_draw_submission_token_records;
    renderer.last_virtual_geometry_indirect_args_buffer = indirect_args_buffer;
    renderer.last_virtual_geometry_indirect_args_count = indirect_args_count;
    renderer.last_virtual_geometry_indirect_submission_buffer = indirect_submission_buffer;
    renderer.last_virtual_geometry_indirect_draw_refs_buffer = indirect_draw_ref_buffer;
    renderer.last_virtual_geometry_indirect_segments_buffer = indirect_segment_buffer;
    Ok(())
}
