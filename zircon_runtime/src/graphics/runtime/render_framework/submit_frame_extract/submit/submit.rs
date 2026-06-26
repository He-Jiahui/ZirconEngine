use std::sync::{Arc, MutexGuard};

use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use crate::graphics::ViewportCameraStackOutputPolicy;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context_from_runtime_frame_extract;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::{update_stats, SharedViewportProductReports};
use super::super::viewport_generation_guard::{
    validate_viewport_generation, viewport_record_mut_after_generation_check,
};
use super::build_runtime_frame::build_runtime_frame;
use super::camera_loop::{submit_camera_loop, CameraLoopOutputPolicy};
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::record_camera_history::record_non_viewport_camera_state_after_success;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;
use super::update_particle_previous_state::update_particle_previous_state_after_success;
use super::update_temporal_camera_history::update_temporal_camera_history_after_success;

pub(in crate::graphics::runtime::render_framework) fn submit_frame_extract(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> Result<(), RenderFrameworkError> {
    submit_frame_extract_with_ui(framework, viewport, extract, None)
}

pub(in crate::graphics::runtime::render_framework) fn submit_frame_extract_with_ui(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "submit_frame_extract");
    let _operation_guard = framework.lock_operation();
    submit_camera_loop(
        framework,
        viewport,
        extract,
        ui,
        submit_selected_camera_frame,
    )
}

fn submit_selected_camera_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &mut Arc<RenderFrameExtract>,
    ui: Option<UiRenderExtract>,
    output_policy: CameraLoopOutputPolicy,
) -> Result<(), RenderFrameworkError> {
    let output_policy = ViewportCameraStackOutputPolicy::from(output_policy);
    let owns_viewport_submission = output_policy.owns_viewport_submission();
    let owns_shared_viewport_products = output_policy.owns_shared_viewport_products();
    let context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context_from_runtime_frame_extract(
            framework,
            viewport,
            extract,
            ui.as_ref(),
        ) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(framework, viewport, &error);
                return Err(error);
            }
        }
    };
    let mut state = framework.lock_state();
    let active_capture =
        owns_shared_viewport_products && begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = {
        crate::profile_scope!("runtime", "render_framework", "prepare_runtime_submission");
        match prepare_runtime_submission(&mut state, viewport, &context) {
            Ok(prepared) => prepared,
            Err(error) => {
                finish_or_fail_capture_after_submission_error(
                    framework,
                    state,
                    viewport,
                    active_capture,
                    &error,
                );
                return Err(error);
            }
        }
    };
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    let mut runtime_frame = build_runtime_frame(ui, &context, prepared, output_policy);
    if owns_shared_viewport_products {
        state.last_virtual_geometry_debug_snapshot =
            runtime_frame.virtual_geometry_debug_snapshot.clone();
    }
    let frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        match state.renderer.render_frame_with_pipeline(
            &runtime_frame,
            context.compiled_pipeline(),
            context.capabilities(),
            resolved_history.current_history_handle(),
            resolved_history.previous_history_available(),
        ) {
            Ok(frame) => frame,
            Err(error) => {
                let error = render_framework_backend_error(error);
                finish_or_fail_capture_after_submission_error(
                    framework,
                    state,
                    viewport,
                    active_capture,
                    &error,
                );
                return Err(error);
            }
        }
    };
    let frame_generation = frame.generation;
    state = finish_active_capture_and_relock(
        framework,
        state,
        active_capture,
        Some(frame_generation),
        None,
    );
    let runtime_feedback = {
        crate::profile_scope!("runtime", "render_framework", "collect_runtime_feedback");
        collect_runtime_feedback(
            &mut state.renderer,
            &context,
            runtime_frame.prepared_runtime_sidebands_mut(),
        )
    };
    let camera_light_grid_report = state.renderer.last_light_grid_report();
    if let Err(error) = validate_viewport_generation(&state, viewport, &context) {
        if !owns_shared_viewport_products {
            fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
        }
        return Err(error);
    }
    if !owns_viewport_submission {
        record_non_viewport_camera_state_after_success(
            &mut state,
            viewport,
            &context,
            &runtime_frame,
            camera_light_grid_report,
            runtime_feedback,
            frame_generation,
            resolved_history.allocated_history(),
        )?;
        return Ok(());
    }
    let record = viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
    record.record_camera_product_reports(
        context.camera_history_key(),
        camera_light_grid_report,
        runtime_frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let record_update = record_submission(
        record,
        &context,
        resolved_history.allocated_history(),
        frame,
        runtime_feedback,
    );
    update_temporal_camera_history_after_success(
        record,
        &runtime_frame,
        context.camera_history_key(),
        true,
    );
    update_particle_previous_state_after_success(
        record,
        &runtime_frame,
        context.camera_history_key(),
    );
    release_previous_history(&mut state.renderer, &record_update);
    if owns_shared_viewport_products {
        let shared_product_reports = SharedViewportProductReports::new(camera_light_grid_report);
        update_stats(
            &mut state,
            &context,
            &record_update,
            frame_generation,
            shared_product_reports,
        );
    }
    crate::profile_counter!(
        "runtime",
        "render_framework.last_frame_generation",
        frame_generation
    );
    Ok(())
}

fn finish_or_fail_capture_after_submission_error(
    framework: &WgpuRenderFramework,
    mut state: MutexGuard<'_, RenderFrameworkState>,
    viewport: RenderViewportHandle,
    active_capture: bool,
    error: &RenderFrameworkError,
) {
    if active_capture {
        drop(finish_active_capture_and_relock(
            framework,
            state,
            active_capture,
            None,
            Some(error.to_string()),
        ));
    } else {
        fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
    }
}

fn fail_pending_capture_after_preflight_error(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = framework.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}
