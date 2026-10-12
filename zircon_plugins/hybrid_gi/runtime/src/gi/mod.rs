#[path = "build_resolve_runtime/mod.rs"]
mod build_resolve_runtime;
#[path = "declarations/mod.rs"]
mod declarations;
#[path = "extract_registration.rs"]
mod extract_registration;
#[path = "gpu_completion.rs"]
mod gpu_completion;
#[cfg(test)]
#[path = "test_sources/tests/stats.rs"]
mod hybrid_gi_render_framework_stats;
#[cfg(test)]
#[path = "../tests/hybrid_gi_scene_prepare_material.rs"]
mod hybrid_gi_scene_prepare_material_fixtures;
#[cfg(test)]
#[path = "tests/hybrid_gi_scene_representation.rs"]
mod hybrid_gi_scene_representation_tests;
#[path = "pending_completion/mod.rs"]
mod pending_completion;
#[path = "plan_ingestion.rs"]
mod plan_ingestion;
#[path = "prepare_frame/mod.rs"]
mod prepare_frame;
#[path = "renderer/mod.rs"]
mod renderer;
#[path = "residency_management/mod.rs"]
mod residency_management;
#[path = "runtime_feedback.rs"]
mod runtime_feedback;
#[path = "scene_inputs.rs"]
mod scene_inputs;
#[path = "scene_representation/mod.rs"]
mod scene_representation;
#[path = "scene_trace_support.rs"]
mod scene_trace_support;
#[path = "snapshot.rs"]
mod snapshot;
#[cfg(test)]
#[path = "tests/test_accessors.rs"]
mod test_accessors;
#[path = "types/mod.rs"]
mod types;

#[cfg(test)]
pub(crate) use declarations::HybridGiProbeResidencyState;
pub(crate) use declarations::HybridGiProbeUpdateRequest;
pub(crate) use declarations::HybridGiRuntimeState;
pub(crate) use gpu_completion::HybridGiGpuCompletion;
pub(super) use renderer::runtime_prepare_collector;
pub(crate) use runtime_feedback::HybridGiRuntimeFeedback;
pub(crate) use scene_inputs::HybridGiSceneInputs;
pub(crate) use scene_representation::HybridGiRuntimeScenePrepareResources;
pub(crate) use scene_representation::HybridGiScenePrepareResourceSamples;
#[cfg(test)]
pub(crate) use scene_representation::{
    HybridGiInputSet, HybridGiSceneRepresentation, HybridGiSurfaceParticipation,
};
pub(crate) use types::*;
