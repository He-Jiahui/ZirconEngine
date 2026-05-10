use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn request_graphics_debugger_capture(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = framework.state.lock().unwrap();
    if !state.viewports.contains_key(&viewport) {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }

    state.graphics_debugger.request_capture(viewport);
    Ok(())
}
