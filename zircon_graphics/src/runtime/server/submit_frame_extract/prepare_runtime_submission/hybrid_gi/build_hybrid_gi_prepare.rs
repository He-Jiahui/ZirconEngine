use crate::runtime::HybridGiRuntimeState;
use crate::types::HybridGiPrepareFrame;

pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) fn build_hybrid_gi_prepare(
    runtime: Option<&HybridGiRuntimeState>,
) -> Option<HybridGiPrepareFrame> {
    runtime.map(HybridGiRuntimeState::build_prepare_frame)
}
