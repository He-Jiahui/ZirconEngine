use std::sync::Arc;

use zircon_runtime::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage, RuntimePrepareCollectorRegistration,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod hybrid_gi;
mod plugin;
mod provider;
mod render_pass_executors;
#[cfg(test)]
pub(crate) mod test_support;

pub use capability::{
    HYBRID_GI_ADVANCED_RENDER_CAPABILITY, HYBRID_GI_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, HybridGiRuntimePlugin,
};
pub use provider::PluginHybridGiRuntimeProvider;

pub(crate) use hybrid_gi::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimeScenePrepareResources,
    HybridGiRuntimeState, HybridGiSceneInputs,
};
use render_pass_executors::{
    hybrid_gi_history_executor, hybrid_gi_resolve_executor, hybrid_gi_scene_prepare_executor,
    hybrid_gi_trace_schedule_executor,
};

pub const PLUGIN_ID: &str = "hybrid_gi";
pub const HYBRID_GI_FEATURE_NAME: &str = "hybrid_gi";
pub const HYBRID_GI_MODULE_NAME: &str = "HybridGiPluginModule";
pub(crate) const HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL: &str =
    "zircon-hybrid-gi-scene-depth-handoff";
pub(crate) const HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE: [u32; 3] = [1, 1, 1];
pub(crate) const HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];
const HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL: &str = "zircon-hybrid-gi-trace-schedule";
const HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        HYBRID_GI_MODULE_NAME,
        "Hybrid global illumination render feature plugin",
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        HYBRID_GI_FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-scene-prepare",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.scene-prepare")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
                HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
                HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS,
            ))
            .read_texture("scene-depth")
            .write_buffer("hybrid-gi-scene"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
                HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
                HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS,
            ))
            .read_buffer("hybrid-gi-scene")
            .write_buffer("hybrid-gi-trace"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer("hybrid-gi-trace")
            .write_texture("hybrid-gi-lighting"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "hybrid-gi-history",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.history")
            .read_texture("hybrid-gi-lighting")
            .write_external_texture("history-global-illumination"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(
            "hybrid-gi.scene-prepare",
            hybrid_gi_scene_prepare_executor,
        ),
        RenderPassExecutorRegistration::new(
            "hybrid-gi.trace-schedule",
            hybrid_gi_trace_schedule_executor,
        ),
        RenderPassExecutorRegistration::new("hybrid-gi.resolve", hybrid_gi_resolve_executor),
        RenderPassExecutorRegistration::new("hybrid-gi.history", hybrid_gi_history_executor),
    ]
}

pub fn runtime_prepare_collector_registration() -> RuntimePrepareCollectorRegistration {
    RuntimePrepareCollectorRegistration::new_collector(
        "hybrid-gi.runtime-prepare",
        crate::hybrid_gi::runtime_prepare_collector(),
    )
}

pub fn hybrid_gi_runtime_provider_registration(
) -> zircon_runtime::graphics::HybridGiRuntimeProviderRegistration {
    zircon_runtime::graphics::HybridGiRuntimeProviderRegistration::new(
        "plugin.hybrid_gi.runtime",
        Arc::new(PluginHybridGiRuntimeProvider),
    )
}

#[cfg(test)]
mod tests;
