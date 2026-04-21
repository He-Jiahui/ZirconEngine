use crate::graphics::runtime::HybridGiRuntimeState;
use crate::graphics::types::HybridGiScenePrepareFrame;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn build_hybrid_gi_scene_prepare(
    runtime: Option<&HybridGiRuntimeState>,
) -> Option<HybridGiScenePrepareFrame> {
    runtime.map(HybridGiRuntimeState::build_scene_prepare_frame)
}
