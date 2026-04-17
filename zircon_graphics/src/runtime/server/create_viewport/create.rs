use zircon_render_server::{RenderServerError, RenderViewportDescriptor, RenderViewportHandle};

use super::super::viewport_record::ViewportRecord;
use super::super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn create_viewport(
    server: &WgpuRenderServer,
    descriptor: RenderViewportDescriptor,
) -> Result<RenderViewportHandle, RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let handle = RenderViewportHandle::new(state.next_viewport_id);
    state.next_viewport_id += 1;
    state
        .viewports
        .insert(handle, ViewportRecord::new(descriptor));
    state.stats.active_viewports = state.viewports.len();
    Ok(handle)
}
