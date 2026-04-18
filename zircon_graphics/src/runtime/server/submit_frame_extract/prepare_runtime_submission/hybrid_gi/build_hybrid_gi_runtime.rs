use crate::runtime::HybridGiRuntimeState;

use super::super::super::frame_submission_context::FrameSubmissionContext;

pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) fn build_hybrid_gi_runtime(
    context: &FrameSubmissionContext,
) -> Option<HybridGiRuntimeState> {
    let extract = context.hybrid_gi_extract.as_ref()?;
    let mut runtime = context
        .previous_hybrid_gi_runtime
        .clone()
        .unwrap_or_default();
    runtime.register_extract(Some(extract));
    if let Some(plan) = context.hybrid_gi_update_plan.as_ref() {
        runtime.ingest_plan(context.predicted_generation, plan);
    }
    Some(runtime)
}
