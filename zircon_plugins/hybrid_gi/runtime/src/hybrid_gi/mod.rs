mod build_resolve_runtime;
mod declarations;
mod extract_payloads;
mod extract_registration;
mod gpu_completion;
#[cfg(test)]
#[path = "test_sources/hybrid_gi_runtime.rs"]
mod hybrid_gi_runtime_tests;
#[cfg(test)]
#[path = "test_sources/hybrid_gi_scene_representation.rs"]
mod hybrid_gi_scene_representation_tests;
#[cfg(test)]
#[path = "test_sources/hybrid_gi_visibility.rs"]
mod hybrid_gi_visibility_tests;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod renderer;
mod residency_management;
mod runtime_feedback;
mod scene_inputs;
mod scene_representation;
mod scene_trace_support;
mod snapshot;
#[cfg(test)]
mod test_accessors;
mod types;

#[cfg(test)]
pub(crate) use declarations::HybridGiProbeResidencyState;
pub(crate) use declarations::HybridGiProbeUpdateRequest;
pub(crate) use declarations::HybridGiRuntimeState;
pub(in crate::hybrid_gi) use extract_payloads::{
    first_hybrid_gi_runtime_probe_payloads, first_hybrid_gi_runtime_trace_region_payloads,
};
pub(crate) use gpu_completion::HybridGiGpuCompletion;
pub(crate) use runtime_feedback::HybridGiRuntimeFeedback;
pub(crate) use scene_inputs::HybridGiSceneInputs;
pub(crate) use scene_representation::HybridGiRuntimeScenePrepareResources;
pub(crate) use scene_representation::HybridGiScenePrepareResourceSamples;
#[cfg(test)]
pub(crate) use scene_representation::{HybridGiInputSet, HybridGiSceneRepresentation};
pub(crate) use types::*;
