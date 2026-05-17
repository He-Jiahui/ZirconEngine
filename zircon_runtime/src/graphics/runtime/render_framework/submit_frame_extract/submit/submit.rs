use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
use super::super::viewport_generation_guard::validate_viewport_generation;
use super::build_runtime_frame::build_runtime_frame;
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;

pub(in crate::graphics::runtime::render_framework) fn submit_frame_extract(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> Result<(), RenderFrameworkError> {
    submit_frame_extract_with_ui(server, viewport, extract, None)
}

pub(in crate::graphics::runtime::render_framework) fn submit_frame_extract_with_ui(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "submit_frame_extract");
    let _operation_guard = server.lock_operation();
    let context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context(server, viewport, &extract, ui.as_ref()) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(server, viewport, &error);
                return Err(error);
            }
        }
    };
    let mut state = server.lock_state();
    let active_capture = begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = {
        crate::profile_scope!("runtime", "render_framework", "prepare_runtime_submission");
        match prepare_runtime_submission(&mut state, viewport, &context) {
            Ok(prepared) => prepared,
            Err(error) => {
                drop(finish_active_capture_and_relock(
                    server,
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
    let runtime_frame = build_runtime_frame(extract, ui, &context, &prepared);
    state.last_virtual_geometry_debug_snapshot =
        runtime_frame.virtual_geometry_debug_snapshot.clone();
    let frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        match state.renderer.render_frame_with_pipeline(
            &runtime_frame,
            context.compiled_pipeline(),
            resolved_history.current_history_handle(),
        ) {
            Ok(frame) => frame,
            Err(error) => {
                let error = render_framework_backend_error(error);
                drop(finish_active_capture_and_relock(
                    server,
                    state,
                    active_capture,
                    None,
                    Some(error.to_string()),
                ));
                return Err(error);
            }
        }
    };
    let frame_generation = frame.generation;
    state = finish_active_capture_and_relock(
        server,
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
    let record_update = record_submission(
        record,
        &context,
        prepared,
        resolved_history.allocated_history(),
        frame,
        runtime_feedback,
    );
    release_previous_history(&mut state.renderer, &record_update);
    update_stats(&mut state, &context, &record_update, frame_generation);
    crate::profile_counter!(
        "runtime",
        "render_framework.last_frame_generation",
        frame_generation
    );
    Ok(())
}

fn fail_pending_capture_after_preflight_error(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = server.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}
