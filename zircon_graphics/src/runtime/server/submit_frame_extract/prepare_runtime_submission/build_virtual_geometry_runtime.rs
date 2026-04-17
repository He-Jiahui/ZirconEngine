use crate::runtime::VirtualGeometryRuntimeState;

use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn build_virtual_geometry_runtime(
    context: &FrameSubmissionContext,
) -> Option<VirtualGeometryRuntimeState> {
    let extract = context.virtual_geometry_extract.as_ref()?;
    let mut runtime = context
        .previous_virtual_geometry_runtime
        .clone()
        .unwrap_or_default();
    runtime.register_extract(Some(extract));
    if let Some(plan) = context.virtual_geometry_page_upload_plan.as_ref() {
        runtime.ingest_plan(context.predicted_generation, plan);
    }
    Some(runtime)
}
