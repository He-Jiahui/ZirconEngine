use zircon_render_server::{RenderServerError, RenderViewportHandle};

use super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn destroy_viewport(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
) -> Result<(), RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let removed = state.viewports.remove(&viewport);
    if removed.is_none() {
        return Err(RenderServerError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    if let Some(history) = removed.and_then(|record| record.history) {
        state.renderer.release_history(history.handle);
    }
    state.stats.active_viewports = state.viewports.len();
    Ok(())
}
