use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
};

use super::store_parts::VirtualGeometryHardwareRasterizationPassStoreParts;

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryHardwareRasterizationPassOutput
{
    source: RenderVirtualGeometryHardwareRasterizationSource,
    record_count: u32,
    buffer: Option<Arc<wgpu::Buffer>>,
    records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
}

impl VirtualGeometryHardwareRasterizationPassOutput {
    pub(super) fn new(
        source: RenderVirtualGeometryHardwareRasterizationSource,
        record_count: u32,
        buffer: Option<Arc<wgpu::Buffer>>,
        records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    ) -> Self {
        Self {
            source,
            record_count,
            buffer,
            records,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_store_parts(
        self,
    ) -> VirtualGeometryHardwareRasterizationPassStoreParts {
        VirtualGeometryHardwareRasterizationPassStoreParts {
            source: self.source,
            record_count: self.record_count,
            buffer: self.buffer,
            records: self.records,
        }
    }
}
