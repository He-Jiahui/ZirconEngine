use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
};

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryHardwareRasterizationPassStoreParts
{
    pub(in crate::virtual_geometry::renderer) source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::virtual_geometry::renderer) record_count: u32,
    pub(in crate::virtual_geometry::renderer) buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) records:
        Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
}
