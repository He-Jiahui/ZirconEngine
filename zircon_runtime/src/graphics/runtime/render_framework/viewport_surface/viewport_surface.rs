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
    crate::profile_scope!("runtime", "render_framework", "replace_viewport_surface");
    let surface = {
        let _operation_guard = framework.lock_operation();
        let state = framework.lock_state();
        if !state.viewports.contains_key(&viewport) {
            return Err(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            });
        }
        state
            .renderer
            .create_framework_viewport_surface(descriptor)
            .map_err(render_framework_backend_error)?
    };

    // The prepared surface is not visible yet. Drain every prior submission so
    // no command can retain the old native surface during publication.
    framework.finish_submission()?;

    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    let retired_histories = state
        .viewports
        .get_mut(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .replace_surface_and_extent(surface, descriptor.size);
    for history in retired_histories {
        state.renderer.release_history(history.handle());
    }
    Ok(())
}

pub(in crate::graphics::runtime::render_framework) fn unbind_viewport_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
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
