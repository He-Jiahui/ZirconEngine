use std::sync::{Arc, MutexGuard};
use std::time::Instant;

use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle, UiRenderSubmission,
};
use crate::graphics::ViewportCameraStackOutputPolicy;

use super::super::super::environment_capture_submission::pump_environment_capture_source_locked;
use super::super::super::frame_profiler::FrameProfiler;
use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::render_framework_state::{
    RenderFrameworkState, ViewportPickFrameSnapshot,
};
use super::super::super::wgpu_render_framework::{WgpuRenderFramework, WgpuRenderFrameworkAccess};
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
use super::completion_error_stats::publish_scene_submission_completion_stats;
use super::publish_viewport_product::publish_viewport_product;
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
    ui: Option<Arc<UiRenderSubmission>>,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "submit_frame_extract");
    framework.dispatch_submission(submit_frame_extract_with_ui_on_core, viewport, extract, ui)
}

fn submit_frame_extract_with_ui_on_core(
    framework: &super::super::super::wgpu_render_framework::WgpuRenderFrameworkCore,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<Arc<UiRenderSubmission>>,
) -> Result<(), RenderFrameworkError> {
    submit_frame_extract_with_ui_locked(framework, viewport, extract, ui)
}

pub(in crate::graphics::runtime::render_framework) fn submit_frame_extract_with_ui_locked(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<Arc<UiRenderSubmission>>,
) -> Result<(), RenderFrameworkError> {
    let submit_started = Instant::now();
    let result = submit_camera_loop(
        framework,
        viewport,
        extract,
        ui,
        &submit_started,
        submit_selected_camera_frame,
    );
    if result.is_ok() {
        pump_environment_capture_source_locked(framework);
    }
    result
}

fn submit_selected_camera_frame(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: &Arc<RenderFrameExtract>,
    ui: Option<Arc<UiRenderSubmission>>,
    submit_started: &Instant,
    output_policy: CameraLoopOutputPolicy,
) -> Result<(), RenderFrameworkError> {
    let output_policy = ViewportCameraStackOutputPolicy::from(output_policy);
    let owns_viewport_submission = output_policy.owns_viewport_submission();
    let owns_shared_viewport_products = output_policy.owns_shared_viewport_products();
    let mut context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context_from_runtime_frame_extract(
            framework,
            viewport,
            extract,
            ui.as_deref(),
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
    let mut runtime_frame = build_runtime_frame(ui, &mut context, prepared, output_policy);
    state
        .renderer
        .set_global_material_mip_bias(context.global_material_mip_bias());
    let viewport_product_requested =
        owns_viewport_submission && state.viewport_products.has_direct_presenter();
    let viewport_capture = (owns_viewport_submission
        && state.viewport_products.requires_async_capture(viewport))
    .then(|| {
        let generation = state.renderer.next_frame_generation();
        state
            .viewports
            .get(&viewport)
            .map(|record| record.async_capture_request(generation))
    })
    .flatten();
    let environment_ibl_bake_reservation = context.take_environment_ibl_bake_reservation();
    let mut frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        let result = if state.renderer.supports_compiled_scene_graph() {
            state
                .renderer
                .render_frame_with_pipeline_async_capture_task_pool_with_environment_ibl_bake_reservation(
                    &runtime_frame,
                    context.compiled_pipeline(),
                    context.capabilities(),
                    crate::core::framework::render::RenderFrameHistoryInput::new(
                        resolved_history.current_history_handle(),
                        resolved_history.previous_history_available(),
                        context.history_invalidation_reason(),
                    ),
                    framework.compute_task_pool(),
                    viewport_capture,
                    environment_ibl_bake_reservation,
                    viewport_product_requested,
                )
        } else {
            state
                .renderer
                .render_frame_direct_submission(&runtime_frame, viewport_product_requested)
        };
        match result {
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
    let frame_generation = frame.generation();
    if let Err(error) = validate_viewport_generation(&state, viewport, &context) {
        finish_or_fail_capture_after_submission_error(
            framework,
            state,
            viewport,
            active_capture,
            &error,
        );
        return Err(error);
    }
    if viewport_product_requested {
        if let Err(error) = publish_viewport_product(&mut state, viewport, &mut frame) {
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
    if owns_shared_viewport_products {
        state.last_virtual_geometry_debug_snapshot =
            runtime_frame.virtual_geometry_debug_snapshot.clone();
    }
    if !owns_viewport_submission {
        record_non_viewport_camera_state_after_success(
            &mut state,
            viewport,
            &context,
            &mut runtime_frame,
            camera_light_grid_report,
            runtime_feedback,
            frame_generation,
            resolved_history.allocated_history(),
        )?;
        return Ok(());
    }
    let record = viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
    let requires_hit_proxies = record.requires_hit_proxies();
    record.record_camera_product_reports(
        context.camera_history_key(),
        camera_light_grid_report,
        runtime_frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let record_update = record_submission(
        record,
        viewport,
        &context,
        resolved_history.allocated_history(),
        frame,
        runtime_feedback,
    )?;
    update_temporal_camera_history_after_success(
        record,
        &mut runtime_frame,
        context.camera_history_key(),
        true,
    );
    update_particle_previous_state_after_success(
        record,
        &mut runtime_frame,
        context.camera_history_key(),
    );
    release_previous_history(&mut state.renderer, &record_update);
    if owns_shared_viewport_products {
        let shared_product_reports = SharedViewportProductReports::new(camera_light_grid_report);
        let frame_profile_write = update_stats(
            &mut state,
            &context,
            &record_update,
            frame_generation,
            FrameProfiler::elapsed_micros(*submit_started),
            shared_product_reports,
        );
        let viewport_record =
            viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
        viewport_record.attach_capture_frame_profile(&frame_profile_write.capture_profile);
        for profile in &frame_profile_write.resolved_gpu_profiles {
            viewport_record.attach_capture_frame_profile(profile);
        }
    }
    if requires_hit_proxies {
        state
            .viewport_pick_frames
            .publish(ViewportPickFrameSnapshot::from_rendered_frame(
                viewport,
                frame_generation,
                &runtime_frame,
                context.virtual_geometry_enabled(),
            ));
    }
    state.last_retained_scene_color_viewport = Some(viewport);
    crate::profile_counter!(
        "runtime",
        "render_framework.last_frame_generation",
        frame_generation
    );
    Ok(())
}

fn finish_or_fail_capture_after_submission_error(
    framework: &dyn WgpuRenderFrameworkAccess,
    mut state: MutexGuard<'_, RenderFrameworkState>,
    viewport: RenderViewportHandle,
    active_capture: bool,
    error: &RenderFrameworkError,
) {
    publish_scene_submission_completion_stats(&mut state);
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
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = framework.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}
