use zircon_render_server::{RenderServerError, RenderViewportDescriptor, RenderViewportHandle};

use super::viewport_record::ViewportRecord;
use super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn create_viewport(
    server: &WgpuRenderServer,
    descriptor: RenderViewportDescriptor,
) -> Result<RenderViewportHandle, RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let handle = RenderViewportHandle::new(state.next_viewport_id);
    state.next_viewport_id += 1;
    state.viewports.insert(
        handle,
        ViewportRecord {
            descriptor,
            pipeline: None,
            quality_profile: None,
            compiled_pipeline: None,
            last_capture: None,
            history: None,
            hybrid_gi_runtime: None,
            virtual_geometry_runtime: None,
        },
    );
    state.stats.active_viewports = state.viewports.len();
    Ok(handle)
}
