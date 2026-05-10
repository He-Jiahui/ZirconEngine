use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportHandle, RenderViewportSurfaceDescriptor,
};

use super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn bind_viewport_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    descriptor: RenderViewportSurfaceDescriptor,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.operation_lock.lock().unwrap();
    let mut state = framework.state.lock().unwrap();
    if !state.viewports.contains_key(&viewport) {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    let surface = state
        .renderer
        .create_viewport_surface(descriptor)
        .map_err(render_framework_backend_error)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    record.bind_surface(surface);
    Ok(())
}

pub(in crate::graphics::runtime::render_framework) fn unbind_viewport_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.operation_lock.lock().unwrap();
    let mut state = framework.state.lock().unwrap();
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    record.unbind_surface();
    Ok(())
}
