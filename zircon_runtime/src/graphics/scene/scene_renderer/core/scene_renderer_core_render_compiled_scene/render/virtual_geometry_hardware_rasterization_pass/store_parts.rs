use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryHardwareRasterizationPassStoreParts
{
    pub(in crate::graphics::scene::scene_renderer::core) source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::graphics::scene::scene_renderer::core) record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) records:
        Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
}
