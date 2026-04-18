use zircon_framework::render::{CapturedFrame, RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::runtime::render_framework) fn capture_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let frame = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .last_capture
        .clone();
    if frame.is_some() {
        state.stats.captured_frames += 1;
    }
    Ok(frame)
}
