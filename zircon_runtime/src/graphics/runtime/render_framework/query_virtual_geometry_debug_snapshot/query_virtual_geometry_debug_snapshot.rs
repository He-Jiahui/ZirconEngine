use crate::core::framework::render::{RenderFrameworkError, RenderVirtualGeometryDebugSnapshot};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
    let state = framework
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let snapshot = state.last_virtual_geometry_debug_snapshot.clone();
    drop(state);
    Ok(snapshot.as_deref().cloned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn virtual_geometry_snapshot_query_clones_payload_after_releasing_state_lock() {
        let source = include_str!("query_virtual_geometry_debug_snapshot.rs");
        let release_marker = concat!("drop(", "state);");
        let owned_clone_marker = concat!("snapshot.as_deref()", ".cloned()");
        let release = source
            .find(release_marker)
            .expect("query must release framework state before cloning the payload");
        let owned_clone = source
            .find(owned_clone_marker)
            .expect("query must preserve the owned public result");

        assert!(release < owned_clone);
    }
}
