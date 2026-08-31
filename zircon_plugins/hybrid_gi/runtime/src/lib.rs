use std::sync::Arc;

use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage, RuntimePrepareCollectorRegistration,
};
use zircon_runtime::render_graph::{
    QueueLane, RenderGraphComputeWorkload, RenderGraphResourceAccessIntent,
    RenderGraphShaderStages, RenderGraphTextureSubresourceRange, RenderResourceSchema,
    RenderTextureExtentPolicy, RenderTextureSchema,
};
use zircon_runtime::rhi::{TextureFormat, TextureUsage};

mod capability;
mod hybrid_gi;
mod plugin;
mod provider;
mod render_pass_executors;
#[cfg(test)]
pub(crate) mod test_support;

pub use capability::{
    HYBRID_GI_ADVANCED_RENDER_CAPABILITY, HYBRID_GI_DECLARATION, HYBRID_GI_RUNTIME_CAPABILITY,
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES,
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
    hybrid_gi_resolve_executor, hybrid_gi_scene_prepare_executor,
    hybrid_gi_trace_schedule_executor, HYBRID_GI_SCENE_BUFFER_MINIMUM_SIZE_BYTES,
    HYBRID_GI_TRACE_BUFFER_MINIMUM_SIZE_BYTES,
};

pub const HYBRID_GI_FEATURE_NAME: &str = "hybrid_gi";
pub const HYBRID_GI_MODULE_NAME: &str = "hybrid_gi.runtime";
pub(crate) const HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL: &str =
    "zircon-hybrid-gi-scene-depth-handoff";
pub(crate) const HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
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
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .write_buffer_with_minimum_size(
                PostProcessGraphResourceNames::HYBRID_GI_SCENE,
                HYBRID_GI_SCENE_BUFFER_MINIMUM_SIZE_BYTES,
            ),
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
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .read_buffer(PostProcessGraphResourceNames::HYBRID_GI_SCENE)
            .write_buffer_with_minimum_size(
                PostProcessGraphResourceNames::HYBRID_GI_TRACE,
                HYBRID_GI_TRACE_BUFFER_MINIMUM_SIZE_BYTES,
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer(PostProcessGraphResourceNames::HYBRID_GI_TRACE)
            .read_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
            .read_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
                hybrid_gi_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
            )
            .read_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
                hybrid_gi_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
            )
            .write_persistent_texture(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING)
            .write_persistent_texture(PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

fn hybrid_gi_history_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        )
        .with_extent(RenderTextureExtentPolicy::View),
    )
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
