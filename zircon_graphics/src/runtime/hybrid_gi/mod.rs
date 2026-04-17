mod extract_registration;
#[cfg(test)]
mod hybrid_gi_probe_residency_state;
mod hybrid_gi_probe_update_request;
mod hybrid_gi_runtime_snapshot;
mod hybrid_gi_runtime_state;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod residency_management;
mod snapshot;
#[cfg(test)]
mod test_accessors;

#[cfg(test)]
pub(crate) use hybrid_gi_probe_residency_state::HybridGiProbeResidencyState;
#[cfg(test)]
pub(crate) use hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;
pub(crate) use hybrid_gi_runtime_state::HybridGiRuntimeState;
