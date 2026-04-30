mod build_resolve_runtime;
mod declarations;
mod extract_payloads;
mod extract_registration;
mod gpu_completion;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod residency_management;
mod runtime_feedback;
mod scene_inputs;
mod scene_representation;
mod scene_trace_support;
mod snapshot;
#[cfg(test)]
mod test_accessors;

#[cfg(test)]
pub(crate) use declarations::HybridGiProbeResidencyState;
pub(crate) use declarations::HybridGiProbeUpdateRequest;
pub(crate) use declarations::HybridGiRuntimeState;
pub(in crate::graphics::runtime::hybrid_gi) use extract_payloads::{
    first_hybrid_gi_runtime_probe_payloads, first_hybrid_gi_runtime_trace_region_payloads,
};
pub(in crate::graphics::runtime) use gpu_completion::HybridGiGpuCompletion;
pub(in crate::graphics::runtime) use runtime_feedback::HybridGiRuntimeFeedback;
pub(in crate::graphics::runtime) use scene_inputs::HybridGiSceneInputs;
pub(in crate::graphics::runtime) use scene_representation::HybridGiRuntimeScenePrepareResources;
pub(crate) use scene_representation::HybridGiScenePrepareResourceSamples;
#[cfg(test)]
pub(crate) use scene_representation::{HybridGiInputSet, HybridGiSceneRepresentation};
