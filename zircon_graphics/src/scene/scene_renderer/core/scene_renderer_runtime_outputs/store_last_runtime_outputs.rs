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
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_count: u32,
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
    renderer.last_virtual_geometry_indirect_args_buffer = indirect_args_buffer;
    renderer.last_virtual_geometry_indirect_args_count = indirect_args_count;
    Ok(())
}
