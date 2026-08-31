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

    #[test]
    fn virtual_geometry_availability_query_does_not_clone_the_payload() {
        let source = include_str!("query_virtual_geometry_debug_snapshot.rs");
        let start = source
            .find("fn query_virtual_geometry_debug_snapshot_available")
            .expect("availability query");
        let end = source[start..]
            .find("#[cfg(test)]")
            .map(|offset| start + offset)
            .expect("availability query end");
        let body = &source[start..end];

        assert!(body.contains("last_virtual_geometry_debug_snapshot.is_some()"));
        assert!(!body.contains(".clone()"));
        assert!(!body.contains(".cloned()"));
    }
}
