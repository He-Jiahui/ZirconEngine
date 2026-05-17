use crate::core::framework::render::{GraphicsDebuggerStatus, RenderFrameworkError};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_graphics_debugger_status(
    framework: &WgpuRenderFramework,
) -> Result<GraphicsDebuggerStatus, RenderFrameworkError> {
    Ok(framework.lock_state().graphics_debugger.status())
}
