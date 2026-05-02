use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryVisBuffer64PassStoreParts {
    pub(in crate::virtual_geometry::renderer) clear_value: u64,
    pub(in crate::virtual_geometry::renderer) entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub(in crate::virtual_geometry::renderer) source: RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::virtual_geometry::renderer) entry_count: u32,
    pub(in crate::virtual_geometry::renderer) buffer: Option<Arc<wgpu::Buffer>>,
}
