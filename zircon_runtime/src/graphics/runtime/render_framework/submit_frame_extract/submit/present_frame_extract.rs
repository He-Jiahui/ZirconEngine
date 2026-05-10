use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
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
    let _operation_guard = framework.operation_lock.lock().unwrap();
    let context = match build_frame_submission_context(framework, viewport, &extract, None) {
        Ok(context) => context,
        Err(error) => {
            fail_pending_capture_after_preflight_error(framework, viewport, &error);
            return Err(error);
        }
    };
    let mut state = framework.state.lock().unwrap();
    let active_capture = begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = match prepare_runtime_submission(&mut state, viewport, &context) {
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
    };
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    let runtime_frame = build_runtime_frame(extract, None, &context, &prepared);
    state.last_virtual_geometry_debug_snapshot =
        runtime_frame.virtual_geometry_debug_snapshot.clone();

    let mut surface =
        {
            let record = state.viewports.get_mut(&viewport).ok_or(
                RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                },
            )?;
            record.take_surface().ok_or_else(|| {
                RenderFrameworkError::Backend("viewport surface is not bound".to_string())
            })?
        };
    let present_result = state.renderer.present_frame_with_pipeline(
        &runtime_frame,
        context.compiled_pipeline(),
        resolved_history.current_history_handle(),
        &mut surface,
    );
    state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked before present")
        .bind_surface(surface);
    let frame_generation = match present_result {
        Ok(generation) => generation,
        Err(error) => {
            let error = render_framework_backend_error(error);
            drop(finish_active_capture_and_relock(
                framework,
                state,
                active_capture,
                None,
                Some(error.to_string()),
            ));
            return Err(error);
        }
    };

    state = finish_active_capture_and_relock(
        framework,
        state,
        active_capture,
        Some(frame_generation),
        None,
    );
    let runtime_feedback = collect_runtime_feedback(&mut state.renderer, &context, &prepared);
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
    Ok(())
}

fn fail_pending_capture_after_preflight_error(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = framework.state.lock().unwrap();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}
