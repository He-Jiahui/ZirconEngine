use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn destroy_viewport(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = server.lock_operation();
    let mut state = server.lock_state();
    let removed = state.viewports.remove(&viewport);
    if removed.is_none() {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    if let Some(history) = removed.and_then(|record| record.into_history()) {
        state.renderer.release_history(history.handle());
    }
    state.graphics_debugger.forget_viewport(viewport);
    state.stats.active_viewports = state.viewports.len();
    Ok(())
}
