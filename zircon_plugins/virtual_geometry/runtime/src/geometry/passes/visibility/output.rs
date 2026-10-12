use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};

use super::store_parts::VirtualGeometryVisBuffer64PassStoreParts;

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryVisBuffer64PassOutput {
    clear_value: u64,
    entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    source: RenderVirtualGeometryVisBuffer64Source,
    entry_count: u32,
    buffer: Option<Arc<wgpu::Buffer>>,
}

impl VirtualGeometryVisBuffer64PassOutput {
    pub(super) fn new(
        clear_value: u64,
        entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
        source: RenderVirtualGeometryVisBuffer64Source,
        entry_count: u32,
        buffer: Option<Arc<wgpu::Buffer>>,
    ) -> Self {
        Self {
            clear_value,
            entries,
            source,
            entry_count,
            buffer,
        }
    }

    pub(in crate::virtual_geometry::renderer) fn into_store_parts(
        self,
    ) -> VirtualGeometryVisBuffer64PassStoreParts {
        VirtualGeometryVisBuffer64PassStoreParts {
            clear_value: self.clear_value,
            entries: self.entries,
            source: self.source,
            entry_count: self.entry_count,
            buffer: self.buffer,
        }
    }
}
