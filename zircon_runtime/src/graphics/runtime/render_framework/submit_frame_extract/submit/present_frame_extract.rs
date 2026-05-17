use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::{
    build_frame_submission_context, validate_camera_surface_present_target,
};
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_present_submission;
use super::super::update_stats::update_stats;
use super::super::viewport_generation_guard::validate_viewport_generation;
use super::build_runtime_frame::build_runtime_frame;
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;

pub(in crate::graphics::runtime::render_framework) fn present_frame_extract(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "present_frame_extract");
    let _operation_guard = framework.lock_operation();
    let context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context(framework, viewport, &extract, None) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(framework, viewport, &error);
                return Err(error);
            }
        }
    };
    if let Err(error) = validate_camera_surface_present_target(&extract.view.camera.target) {
        fail_pending_capture_after_preflight_error(framework, viewport, &error);
        return Err(error);
    }
    let mut state = framework.lock_state();
    if let Err(error) = preflight_bound_surface(&mut state, viewport) {
        fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
        return Err(error);
    }
    let active_capture = begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = {
        crate::profile_scope!("runtime", "render_framework", "prepare_runtime_submission");
        match prepare_runtime_submission(&mut state, viewport, &context) {
            Ok(prepared) => prepared,
            Err(error) => {
                drop(finish_active_capture_and_relock(
                    framework,
                    state,
                    active_capture,
                    None,
                    Some(error.to_string()),
                ));
                return Err(error);
            }
        }
    };
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    let runtime_frame = build_runtime_frame(extract, None, &context, &prepared);
    state.last_virtual_geometry_debug_snapshot =
        runtime_frame.virtual_geometry_debug_snapshot.clone();

    let present_result: Result<_, RenderFrameworkError> = {
        let RenderFrameworkState {
            renderer,
            viewports,
            ..
        } = &mut *state;
        let Some(record) = viewports.get_mut(&viewport) else {
            return finish_capture_and_return_error(
                framework,
                state,
                active_capture,
                RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                },
            );
        };
        let Some(mut surface_lease) = record.lease_surface() else {
            return finish_capture_and_return_error(
                framework,
                state,
                active_capture,
                unsupported_viewport_surface_present(),
            );
        };
        let present_result = {
            crate::profile_scope!("runtime", "render_framework", "present_frame_with_pipeline");
            renderer.present_frame_with_pipeline(
                &runtime_frame,
                context.compiled_pipeline(),
                resolved_history.current_history_handle(),
                surface_lease.value_mut(),
            )
        };
        surface_lease.restore();
        present_result.map_err(render_framework_backend_error)
    };
    let frame_generation = match present_result {
        Ok(generation) => generation,
        Err(error) => {
            return finish_capture_and_return_error(framework, state, active_capture, error);
        }
    };

    state = finish_active_capture_and_relock(
        framework,
        state,
        active_capture,
        Some(frame_generation),
        None,
    );
    let runtime_feedback = {
        crate::profile_scope!("runtime", "render_framework", "collect_runtime_feedback");
        collect_runtime_feedback(&mut state.renderer, &context, &prepared)
    };
    validate_viewport_generation(&state, viewport, &context)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport generation checked above");
    let record_update = record_present_submission(
        record,
        &context,
        prepared,
        resolved_history.allocated_history(),
        frame_generation,
        runtime_feedback,
    );
    release_previous_history(&mut state.renderer, &record_update);
    update_stats(&mut state, &context, &record_update, frame_generation);
    crate::profile_counter!(
        "runtime",
        "render_framework.last_present_generation",
        frame_generation
    );
    Ok(())
}

fn preflight_bound_surface(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    let record = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?;
    if record.has_surface() {
        return Ok(());
    }
    Err(unsupported_viewport_surface_present())
}

fn unsupported_viewport_surface_present() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: "viewport surface present".to_string(),
    }
}

fn finish_capture_and_return_error(
    framework: &WgpuRenderFramework,
    state: std::sync::MutexGuard<'_, RenderFrameworkState>,
    active_capture: bool,
    error: RenderFrameworkError,
) -> Result<(), RenderFrameworkError> {
    drop(finish_active_capture_and_relock(
        framework,
        state,
        active_capture,
        None,
        Some(error.to_string()),
    ));
    Err(error)
}

fn fail_pending_capture_after_preflight_error(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = framework.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}
