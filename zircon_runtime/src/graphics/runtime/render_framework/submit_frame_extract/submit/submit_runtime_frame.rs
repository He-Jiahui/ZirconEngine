use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;

pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    mut frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    let context =
        build_frame_submission_context(server, viewport, &frame.extract, frame.ui.as_ref())?;
    apply_effective_advanced_extracts_to_runtime_frame(&mut frame, &context);
    let mut state = server.state.lock().unwrap();
    let prepared = prepare_runtime_submission(&mut state, viewport, &context);
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    state.last_virtual_geometry_debug_snapshot = frame.virtual_geometry_debug_snapshot.clone();
    let frame = state
        .renderer
        .render_frame_with_pipeline(
            &frame,
            context.compiled_pipeline(),
            resolved_history.current_history_handle(),
        )
        .map_err(render_framework_backend_error)?;
    let frame_generation = frame.generation;
    let runtime_feedback = collect_runtime_feedback(&mut state.renderer, &context, &prepared);
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
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
    Ok(())
}

fn apply_effective_advanced_extracts_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.extract.geometry.virtual_geometry = context.virtual_geometry_extract().cloned();
    if !context.hybrid_gi_enabled() {
        frame.extract.lighting.hybrid_global_illumination = None;
    }
}
