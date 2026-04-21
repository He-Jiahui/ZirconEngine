use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
use super::collect_gpu_completions::collect_gpu_completions;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;

pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    let context =
        build_frame_submission_context(server, viewport, &frame.extract, frame.ui.as_ref())?;
    let prepared = prepare_runtime_submission(&context);

    let mut state = server.state.lock().unwrap();
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    let runtime_frame = frame
        .with_hybrid_gi_prepare(prepared.hybrid_gi_prepare.clone())
        .with_hybrid_gi_resolve_runtime(prepared.hybrid_gi_resolve_runtime.clone())
        .with_virtual_geometry_prepare(prepared.virtual_geometry_prepare.clone());
    let frame = state
        .renderer
        .render_frame_with_pipeline(
            &runtime_frame,
            &context.compiled_pipeline,
            resolved_history.current_history_handle,
        )
        .map_err(render_framework_backend_error)?;
    let frame_generation = frame.generation;
    let gpu_completions = collect_gpu_completions(&mut state.renderer);
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    let record_update = record_submission(
        record,
        &context,
        prepared,
        resolved_history.allocated_history,
        frame,
        gpu_completions.hybrid_gi_completion,
        gpu_completions.virtual_geometry_completion,
    );
    release_previous_history(&mut state.renderer, &record_update);
    update_stats(&mut state, &context, &record_update, frame_generation);
    Ok(())
}
