use crate::core::framework::render::{RenderFrameworkError, RenderVirtualGeometryDebugSnapshot};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
    let state = framework.lock_state();
    let snapshot = state.last_virtual_geometry_debug_snapshot.clone();
    drop(state);
    Ok(snapshot.as_deref().cloned())
}

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot_available(
    framework: &WgpuRenderFramework,
) -> Result<bool, RenderFrameworkError> {
    let state = framework.lock_state();
    Ok(state.last_virtual_geometry_debug_snapshot.is_some())
}

#[cfg(test)]
#[path = "tests/query_virtual_geometry_debug_snapshot.rs"]
mod tests;
