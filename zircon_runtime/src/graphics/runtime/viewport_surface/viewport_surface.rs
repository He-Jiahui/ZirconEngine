//! 表面绑定在操作锁下改变视口呈现能力，新尺寸会触发历史释放和图重建。
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
    let prepared_surface = {
        let _operation_guard = framework.lock_operation();
        let state = framework.lock_state();
        let record =
            state
                .viewports
                .get(&viewport)
                .ok_or(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                })?;
        if record.has_native_surface_target(descriptor.target) {
            None
        } else {
            Some(
                state
                    .renderer
                    .create_framework_viewport_surface(descriptor)
                    .map_err(render_framework_backend_error)?,
            )
        }
    };

    // Never drain while holding the operation/state lock: the worker needs both to finish.
    framework.drain_submissions()?;

    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    let device_epoch = state.renderer.virtual_geometry_device_epoch().raw_parts();
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    let result = match prepared_surface {
        Some(surface) => Ok(surface),
        None => {
            let surface = record
                .take_surface_for_native_target(descriptor.target)
                .ok_or_else(|| {
                    RenderFrameworkError::Backend(
                        "surface target changed while draining submissions".to_string(),
                    )
                })?;
            surface.reconfigure_for_target(descriptor, device_epoch)
        }
    };
    let (retired_histories, failure) = match result {
        Ok(surface) => (
            record.replace_surface_and_extent(surface, descriptor.size),
            None,
        ),
        Err(error) => (record.retire_failed_surface_reconfigure(), Some(error)),
    };
    for history in retired_histories {
        state.renderer.release_history(history.handle());
    }
    failure.map_or(Ok(()), |error| Err(render_framework_backend_error(error)))
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
