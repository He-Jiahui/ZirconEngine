use zircon_framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::runtime::render_framework) fn destroy_viewport(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let removed = state.viewports.remove(&viewport);
    if removed.is_none() {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    if let Some(history) = removed.and_then(|record| record.history) {
        state.renderer.release_history(history.handle);
    }
    state.stats.active_viewports = state.viewports.len();
    Ok(())
}
