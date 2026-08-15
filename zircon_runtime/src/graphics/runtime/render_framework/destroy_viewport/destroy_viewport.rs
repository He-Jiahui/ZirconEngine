use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn destroy_viewport(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    let removed = state.viewports.remove(&viewport);
    if removed.is_none() {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    if let Some(record) = removed {
        for history in record.into_histories() {
            state.renderer.release_history(history.handle());
        }
    }
    state.viewport_products.remove(viewport);
    state.graphics_debugger.forget_viewport(viewport);
    state.stats.active_viewports = state.viewports.len();
    Ok(())
}
