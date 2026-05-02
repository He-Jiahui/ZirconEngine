use crate::core::framework::render::{RenderFrameworkError, RenderVirtualGeometryDebugSnapshot};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot(
    server: &WgpuRenderFramework,
) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
    Ok(server
        .state
        .lock()
        .unwrap()
        .last_virtual_geometry_debug_snapshot
        .clone())
}
