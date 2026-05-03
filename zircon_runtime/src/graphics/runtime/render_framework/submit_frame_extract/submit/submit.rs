use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
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
    let context = build_frame_submission_context(server, viewport, &extract, ui.as_ref())?;
    let mut state = server.state.lock().unwrap();
    let prepared = prepare_runtime_submission(&mut state, viewport, &context);
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    let runtime_frame = build_runtime_frame(extract, ui, &context, &prepared);
    state.last_virtual_geometry_debug_snapshot =
        runtime_frame.virtual_geometry_debug_snapshot.clone();
    let frame = state
        .renderer
        .render_frame_with_pipeline(
            &runtime_frame,
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
