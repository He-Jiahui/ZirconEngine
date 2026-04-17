use crate::runtime::HybridGiRuntimeState;
use crate::types::HybridGiPrepareFrame;

pub(super) fn build_hybrid_gi_prepare(
    runtime: Option<&HybridGiRuntimeState>,
) -> Option<HybridGiPrepareFrame> {
    runtime.map(HybridGiRuntimeState::build_prepare_frame)
}
