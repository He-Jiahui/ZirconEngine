use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportHandle, RenderVisibleSpatialQuerySnapshot,
};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_visible_spatial_snapshot(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<Option<RenderVisibleSpatialQuerySnapshot>, RenderFrameworkError> {
    let state = framework.lock_state();
    let snapshot = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .visible_spatial_query();
    drop(state);
    Ok(snapshot)
}

#[cfg(test)]
#[path = "tests/query_visible_spatial_snapshot.rs"]
mod tests;
