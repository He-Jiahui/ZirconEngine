use crate::graphics::runtime::VirtualGeometryRuntimeState;

use super::super::super::frame_submission_context::FrameSubmissionContext;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn build_virtual_geometry_runtime(
    context: &FrameSubmissionContext,
) -> Option<VirtualGeometryRuntimeState> {
    let extract = context.virtual_geometry_extract()?;
    let mut runtime = context
        .previous_virtual_geometry_runtime()
        .cloned()
        .unwrap_or_default();
    runtime.register_extract(Some(extract));
    if let Some(plan) = context.virtual_geometry_page_upload_plan() {
        runtime.ingest_plan(context.predicted_generation(), plan);
    }
    Some(runtime)
}
