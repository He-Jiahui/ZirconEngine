use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
    RenderPassStage, RuntimePrepareCollectorContext, RuntimePrepareCollectorRegistration,
};
use zircon_runtime::render_graph::{
    QueueLane, RenderBufferSchema, RenderGraphComputeWorkload, RenderResourceSchema,
};
use zircon_runtime::rhi::{BufferUsage, RenderDeviceFeature};

#[path = "capability.rs"]
mod capability;
#[path = "plugin.rs"]
mod plugin;
#[path = "provider.rs"]
mod provider;
#[path = "executors.rs"]
mod render_pass_executors;
#[cfg(test)]
#[path = "test_support/tests/mod.rs"]
pub(crate) mod test_support;
#[path = "geometry/mod.rs"]
mod virtual_geometry;

use render_pass_executors::{
    virtual_geometry_debug_overlay_executor, virtual_geometry_indirect_args_executor,
    virtual_geometry_node_cluster_cull_executor, virtual_geometry_page_feedback_executor,
    virtual_geometry_prepare_executor, virtual_geometry_visbuffer_executor,
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
const VIRTUAL_GEOMETRY_NODE_CLUSTER_CULL_DISPATCH_GROUPS: [u32; 3] = [64, 1, 1];
const VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY: u64 = 4096;
const VIRTUAL_GEOMETRY_GRAPH_BUFFER_SIZE_BYTES: u64 =
    (VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY + 1) * std::mem::size_of::<u32>() as u64;
const VIRTUAL_GEOMETRY_INDIRECT_ARGS_SIZE_BYTES: u64 = std::mem::size_of::<u32>() as u64
    + VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY * 4 * std::mem::size_of::<u32>() as u64;
const VIRTUAL_GEOMETRY_INDIRECT_AUTHORITY_SIZE_BYTES: u64 =
    VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY * 14 * std::mem::size_of::<u32>() as u64;

fn virtual_geometry_graph_buffer_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        VIRTUAL_GEOMETRY_GRAPH_BUFFER_SIZE_BYTES,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ))
}

fn virtual_geometry_visbuffer64_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        (1 + VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY * 2) * std::mem::size_of::<u32>() as u64,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ))
}

fn virtual_geometry_indirect_args_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        VIRTUAL_GEOMETRY_INDIRECT_ARGS_SIZE_BYTES,
        BufferUsage::STORAGE
            | BufferUsage::INDIRECT
            | BufferUsage::COPY_SRC
            | BufferUsage::COPY_DST,
    ))
}

fn virtual_geometry_indirect_segments_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        (4 + VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY * 12) * 4,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ))
}

fn virtual_geometry_indirect_authority_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        VIRTUAL_GEOMETRY_INDIRECT_AUTHORITY_SIZE_BYTES,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ))
}

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
    let mut feature = RenderFeatureDescriptor::new(
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
            .write_buffer_with_schema(
                "virtual-geometry-page-requests",
                virtual_geometry_graph_buffer_schema(),
            ),
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
            .read_buffer_with_schema(
                "virtual-geometry-page-requests",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-visible-clusters",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-visible-candidate-indices",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-nonresident-page-requests",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-visbuffer64",
                virtual_geometry_visbuffer64_schema(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-indirect-args",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("virtual-geometry.indirect-args")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-virtual-geometry-indirect-args",
                [64, 1, 1],
                [64, 1, 1],
            ))
            .read_buffer_with_schema(
                "virtual-geometry-visible-candidate-indices",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-indirect-args",
                virtual_geometry_indirect_args_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-indirect-authority",
                virtual_geometry_indirect_authority_schema(),
            )
            .write_buffer_with_schema(
                "virtual-geometry-indirect-segments",
                virtual_geometry_indirect_segments_schema(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-page-feedback",
                QueueLane::AsyncCopy,
            )
            .with_executor_id("virtual-geometry.page-feedback")
            .read_buffer_with_schema(
                "virtual-geometry-nonresident-page-requests",
                virtual_geometry_graph_buffer_schema(),
            )
            .write_external_buffer("virtual-geometry-feedback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-visbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.visbuffer")
            .read_buffer_with_schema(
                "virtual-geometry-visible-candidate-indices",
                virtual_geometry_graph_buffer_schema(),
            )
            .read_buffer_with_schema(
                "virtual-geometry-visible-clusters",
                virtual_geometry_graph_buffer_schema(),
            )
            .read_buffer_with_schema(
                "virtual-geometry-indirect-args",
                virtual_geometry_indirect_args_schema(),
            )
            .read_buffer_with_schema(
                "virtual-geometry-indirect-authority",
                virtual_geometry_indirect_authority_schema(),
            )
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "virtual-geometry-debug-overlay",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.debug-overlay")
            .read_buffer_with_schema(
                "virtual-geometry-visible-clusters",
                virtual_geometry_graph_buffer_schema(),
            )
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry);
    use zircon_runtime::graphics::{
        RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
        RenderFeatureResourceWriteMode,
    };
    use zircon_runtime::render_graph::{
        RenderGraphBufferRange, RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
        RenderGraphResourceAccessRange, RenderGraphShaderStages,
    };
    fn schema(words: u64, indirect: bool) -> RenderResourceSchema {
        let usage = BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST;
        RenderResourceSchema::buffer(RenderBufferSchema::new(
            words * 4,
            if indirect {
                usage | BufferUsage::INDIRECT
            } else {
                usage
            },
        ))
    }
    fn resource(
        name: &str,
        words: u64,
        indirect: bool,
        write: bool,
    ) -> RenderFeatureResourceDescriptor {
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::Overlay,
            "resource-contract",
            QueueLane::AsyncCompute,
        );
        let mut value = if write {
            pass.write_buffer_with_schema(name, schema(words, indirect))
        } else {
            pass.read_buffer_with_schema(name, schema(words, indirect))
        }
        .resources
        .remove(0);
        value.access_metadata = Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(words * 4))),
            if write {
                RenderGraphResourceAccessIntent::StorageBufferReadWrite {
                    stages: RenderGraphShaderStages::COMPUTE,
                }
            } else {
                RenderGraphResourceAccessIntent::StorageBufferRead {
                    stages: RenderGraphShaderStages::COMPUTE,
                }
            },
        ));
        value
    }
    let outputs = [
        resource("virtual-geometry-indexed-args", 65536 * 5, true, true),
        resource("virtual-geometry-indexed-sidecar", 65536 * 32, false, true),
        resource("virtual-geometry-indexed-status", 65536, false, true),
        resource(
            "virtual-geometry-admitted-run-slots",
            65536 + 2,
            false,
            true,
        ),
    ];
    // The original six public descriptors and executor registrations retain
    // their names and order. This seventh descriptor is a real compute pass.
    feature.stage_passes[2]
        .resources
        .extend(outputs.iter().cloned());
    feature.stage_passes[1].resources.push(resource(
        "virtual-geometry-cull-decisions",
        8 + 4096 * 33,
        false,
        true,
    ));
    let mut terminal = RenderFeaturePassDescriptor::new(
        RenderPassStage::Overlay,
        "virtual-geometry-terminal-execution",
        QueueLane::AsyncCompute,
    )
    .with_executor_id("virtual-geometry.indirect-args")
    .with_compute_workload(RenderGraphComputeWorkload::fixed(
        "virtual-geometry-terminal",
        [1, 1, 1],
        [1, 1, 1],
    ));
    terminal.flags.allow_culling = false;
    feature.stage_passes[2].resources.push(resource(
        "virtual-geometry-cull-decisions",
        8 + 4096 * 33,
        false,
        true,
    ));
    terminal.resources.push(resource(
        "virtual-geometry-cull-decisions",
        8 + 4096 * 33,
        false,
        true,
    ));
    for mut read in outputs.iter().cloned() {
        read.access = RenderFeatureResourceAccess::Read;
        read.write_mode = RenderFeatureResourceWriteMode::Attachment;
        read.access_metadata.as_mut().unwrap().intent =
            RenderGraphResourceAccessIntent::StorageBufferRead {
                stages: RenderGraphShaderStages::COMPUTE,
            };
        terminal.resources.push(read);
    }
    terminal = terminal
        .read_buffer_with_schema(
            "virtual-geometry-visible-candidate-indices",
            virtual_geometry_graph_buffer_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-visible-clusters",
            virtual_geometry_graph_buffer_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-nonresident-page-requests",
            virtual_geometry_graph_buffer_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-visbuffer64",
            virtual_geometry_visbuffer64_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-indirect-args",
            virtual_geometry_indirect_args_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-indirect-authority",
            virtual_geometry_indirect_authority_schema(),
        )
        .read_buffer_with_schema(
            "virtual-geometry-indirect-segments",
            virtual_geometry_indirect_segments_schema(),
        );
    terminal.resources.extend([
        resource(
            "virtual-geometry-execution-args",
            1 + 65536 * 4,
            false,
            true,
        ),
        resource(
            "virtual-geometry-execution-authority",
            65536 * 14,
            false,
            true,
        ),
        resource(
            "virtual-geometry-execution-submission",
            1 + 65536 * 17,
            false,
            true,
        ),
        resource(
            "virtual-geometry-execution-hardware-metadata",
            1 + 65536 * 18,
            false,
            true,
        ),
        resource(
            "virtual-geometry-execution-selected-clusters",
            1 + 4096 * 8,
            false,
            true,
        ),
    ]);
    feature.stage_passes.push(terminal);
    let mut indirect = outputs[0].clone();
    indirect.access = RenderFeatureResourceAccess::Read;
    indirect.write_mode = RenderFeatureResourceWriteMode::Attachment;
    indirect.access_metadata.as_mut().unwrap().intent = RenderGraphResourceAccessIntent::Indirect;
    feature.with_ordered_buffer_replay(
        vec![
            "mesh.opaque".into(),
            "deferred.gbuffer".into(),
            "mesh.transparent".into(),
        ],
        "virtual-geometry-indirect-args",
        "virtual-geometry-terminal-execution",
        indirect,
        outputs[3].clone(),
    )
}

struct VirtualGeometryVisbufferExecutor;

impl RenderPassExecutor for VirtualGeometryVisbufferExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        virtual_geometry_visbuffer_executor(context)
    }

    fn required_device_features(&self) -> &[RenderDeviceFeature] {
        &[RenderDeviceFeature::IndirectFirstInstance]
    }
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
            "virtual-geometry.indirect-args",
            virtual_geometry_indirect_args_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.page-feedback",
            virtual_geometry_page_feedback_executor,
        ),
        RenderPassExecutorRegistration::new_executor(
            "virtual-geometry.visbuffer",
            Arc::new(VirtualGeometryVisbufferExecutor),
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
#[path = "tests/cases.rs"]
mod tests;
