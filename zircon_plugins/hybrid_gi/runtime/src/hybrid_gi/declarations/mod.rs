#[cfg(test)]
mod hybrid_gi_probe_residency_state;
mod hybrid_gi_probe_update_request;
mod hybrid_gi_runtime_snapshot;
mod hybrid_gi_runtime_state;

#[cfg(test)]
pub(crate) use hybrid_gi_probe_residency_state::HybridGiProbeResidencyState;
pub(crate) use hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;
pub(super) use hybrid_gi_runtime_snapshot::HybridGiRuntimeSnapshot;
pub(crate) use hybrid_gi_runtime_state::HybridGiRuntimeState;
pub(in crate::hybrid_gi) use hybrid_gi_runtime_state::{
    HybridGiRuntimeProbeSceneData, HybridGiRuntimeTraceRegionSceneData,
};
