use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutorRegistration, RenderPassStage, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod plugin;
mod provider;
mod render_pass_executors;
#[cfg(test)]
pub(crate) mod test_support;
mod virtual_geometry;

use render_pass_executors::{
    virtual_geometry_debug_overlay_executor, virtual_geometry_node_cluster_cull_executor,
    virtual_geometry_page_feedback_executor, virtual_geometry_prepare_executor,
    virtual_geometry_visbuffer_executor,
};
use std::sync::Arc;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES,
    VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY, VIRTUAL_GEOMETRY_DECLARATION,
    VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, virtual_geometry_source_descriptor, VirtualGeometryRuntimePlugin,
    VIRTUAL_GEOMETRY_DIST_CRATE_NAME, VIRTUAL_GEOMETRY_DIST_RUNTIME_ENTRY,
    VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_ID, VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_TOKEN,
    VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_WGSL_INCLUDE,
};
pub use provider::PluginVirtualGeometryRuntimeProvider;

pub const VIRTUAL_GEOMETRY_FEATURE_NAME: &str = "virtual_geometry";
pub const VIRTUAL_GEOMETRY_MODULE_NAME: &str = "virtual_geometry.runtime";
const VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_PIPELINE_LABEL: &str =
    "zircon-virtual-geometry-node-cluster-cull";
const VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
const VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

pub fn virtual_geometry_runtime_provider_registration(
) -> zircon_runtime::graphics::VirtualGeometryRuntimeProviderRegistration {
    zircon_runtime::graphics::VirtualGeometryRuntimeProviderRegistration::new(
        PLUGIN_ID,
        Arc::new(PluginVirtualGeometryRuntimeProvider),
    )
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        VIRTUAL_GEOMETRY_MODULE_NAME,
        "Virtual geometry render feature plugin",
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        VIRTUAL_GEOMETRY_FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .write_buffer("virtual-geometry-page-requests"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-node-cluster-cull",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("virtual-geometry.node-cluster-cull")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_PIPELINE_LABEL,
                VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_WORKGROUP_SIZE,
                VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_DISPATCH_GROUPS,
            ))
            .read_buffer("virtual-geometry-page-requests")
            .write_buffer("virtual-geometry-visible-clusters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-page-feedback",
                QueueLane::AsyncCopy,
            )
            .with_executor_id("virtual-geometry.page-feedback")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_external_buffer("virtual-geometry-feedback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-visbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.visbuffer")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "virtual-geometry-debug-overlay",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.debug-overlay")
            .read_buffer("virtual-geometry-visible-clusters")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            virtual_geometry_prepare_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.node-cluster-cull",
            virtual_geometry_node_cluster_cull_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.page-feedback",
            virtual_geometry_page_feedback_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.visbuffer",
            virtual_geometry_visbuffer_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.debug-overlay",
            virtual_geometry_debug_overlay_executor,
        ),
    ]
}

pub fn runtime_prepare_collector_registration() -> RuntimePrepareCollectorRegistration {
    RuntimePrepareCollectorRegistration::new(
        "virtual-geometry.runtime-prepare",
        virtual_geometry_runtime_prepare_collector,
    )
}

fn virtual_geometry_runtime_prepare_collector(
    context: &mut RuntimePrepareCollectorContext<'_>,
) -> Result<
    zircon_runtime::core::framework::render::RenderPluginRendererOutputs,
    zircon_runtime::graphics::GraphicsError,
> {
    Ok(crate::virtual_geometry::runtime_prepare_renderer_outputs(
        context,
    ))
}

#[cfg(test)]
mod tests;
