use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryRenderPathOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_clear_value: u64,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_entry_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) debug_snapshot:
        Option<RenderVirtualGeometryDebugSnapshot>,
}
