use crate::core::framework::render::{CapturedFrame, RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn capture_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "capture_frame");
    let _operation_guard = server.lock_operation();
    let mut state = server.lock_state();
    let frame = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .last_capture()
        .cloned();
    if frame.is_some() {
        state.stats.captured_frames += 1;
    }
    Ok(frame)
}
