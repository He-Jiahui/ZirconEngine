use crate::graphics::runtime::HybridGiRuntimeState;

use super::super::super::frame_submission_context::FrameSubmissionContext;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn build_hybrid_gi_runtime(
    context: &FrameSubmissionContext,
) -> Option<HybridGiRuntimeState> {
    let extract = context.hybrid_gi_extract()?;
    let scene_inputs = context.hybrid_gi_scene_inputs();
    let mut runtime = context
        .previous_hybrid_gi_runtime()
        .cloned()
        .unwrap_or_default();
    runtime.register_scene_extract(
        Some(extract),
        scene_inputs.meshes(),
        scene_inputs.directional_lights(),
        scene_inputs.point_lights(),
        scene_inputs.spot_lights(),
    );
    if let Some(plan) = context.hybrid_gi_update_plan() {
        runtime.ingest_plan(context.predicted_generation(), plan);
    }
    Some(runtime)
}
