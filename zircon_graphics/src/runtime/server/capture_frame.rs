use zircon_render_server::{CapturedFrame, RenderServerError, RenderViewportHandle};

use super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn capture_frame(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
) -> Result<Option<CapturedFrame>, RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let frame = state
        .viewports
        .get(&viewport)
        .ok_or(RenderServerError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .last_capture
        .clone();
    if frame.is_some() {
        state.stats.captured_frames += 1;
    }
    Ok(frame)
}
