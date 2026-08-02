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
mod tests {
    #[test]
    fn visible_spatial_snapshot_query_releases_framework_state_after_copying_snapshot_handle() {
        let source = include_str!("query_visible_spatial_snapshot.rs");
        let release = source
            .find("drop(state);")
            .expect("query releases framework state");
        let return_result = source.find("Ok(snapshot)").expect("query returns snapshot");

        assert!(release < return_result);
    }
}
