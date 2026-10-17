#[cfg(test)]
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use zircon_runtime::graphics::RenderPassExecutionContext;
#[cfg(test)]
use zircon_runtime::graphics::{RenderPassExecutor, RenderPassExecutorRegistration};
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

#[path = "executors/gpu.rs"]
mod gpu;
#[path = "executors/indirect.rs"]
mod indirect;
#[path = "executors/native_probe.rs"]
pub(super) mod native_probe;

#[cfg(test)]
pub(crate) type IndirectReadbackWords = indirect::IndirectReadbackWords;

#[cfg(test)]
pub(crate) type CullGpuReadbackWords = gpu::CullGpuReadbackWords;

#[derive(Clone, Copy, Debug)]
struct RenderPassExecutorContract {
    pass_name: &'static str,
    executor_id: &'static str,
    declared_queue: QueueLane,
    flags: PassFlags,
    resources: &'static [ExpectedResource],
}

#[derive(Clone, Copy, Debug)]
struct ExpectedResource {
    name: &'static str,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
}

impl ExpectedResource {
    const fn new(
        name: &'static str,
        kind: RenderGraphResourceKind,
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self { name, kind, access }
    }

    fn description(self) -> String {
        describe_resource(self.name, self.kind, self.access)
    }
}

const PREPARE_RESOURCES: &[ExpectedResource] = &[ExpectedResource::new(
    "virtual-geometry-page-requests",
    RenderGraphResourceKind::TransientBuffer,
    RenderGraphResourceAccessKind::Write,
)];

const NODE_CLUSTER_CULL_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-page-requests",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-visible-clusters",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-visible-candidate-indices",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-nonresident-page-requests",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-visbuffer64",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
];

const PAGE_FEEDBACK_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-nonresident-page-requests",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-feedback",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
];

const VISBUFFER_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-visible-clusters",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-authority",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-depth",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
];

const DEBUG_OVERLAY_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-visible-clusters",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-color",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-color",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
];

const INDIRECT_ARGS_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-visible-candidate-indices",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-authority",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-indexed-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-indexed-sidecar",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-indexed-status",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-admitted-run-slots",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
];

const PREPARE_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-prepare",
    executor_id: "virtual-geometry.prepare",
    declared_queue: QueueLane::Graphics,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: PREPARE_RESOURCES,
};

const NODE_CLUSTER_CULL_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-node-cluster-cull",
    executor_id: "virtual-geometry.node-cluster-cull",
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: NODE_CLUSTER_CULL_RESOURCES,
};

const PAGE_FEEDBACK_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-page-feedback",
    executor_id: "virtual-geometry.page-feedback",
    declared_queue: QueueLane::AsyncCopy,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: PAGE_FEEDBACK_RESOURCES,
};

const VISBUFFER_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-visbuffer",
    executor_id: "virtual-geometry.visbuffer",
    declared_queue: QueueLane::Graphics,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: VISBUFFER_RESOURCES,
};

const DEBUG_OVERLAY_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-debug-overlay",
    executor_id: "virtual-geometry.debug-overlay",
    declared_queue: QueueLane::Graphics,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: DEBUG_OVERLAY_RESOURCES,
};

const TERMINAL_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-indexed-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indexed-sidecar",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indexed-status",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-admitted-run-slots",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-indirect-authority",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-execution-args",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-execution-authority",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-execution-submission",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-execution-hardware-metadata",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "virtual-geometry-visible-candidate-indices",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-visible-clusters",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-nonresident-page-requests",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-visbuffer64",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "virtual-geometry-execution-selected-clusters",
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
];
const TERMINAL_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-terminal-execution",
    executor_id: "virtual-geometry.indirect-args",
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: false,
        has_side_effects: false,
    },
    resources: TERMINAL_RESOURCES,
};

const INDIRECT_ARGS_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "virtual-geometry-indirect-args",
    executor_id: "virtual-geometry.indirect-args",
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: INDIRECT_ARGS_RESOURCES,
};

pub(crate) fn virtual_geometry_prepare_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &PREPARE_CONTRACT)?;
    gpu::execute_prepare(context.require_gpu()?)
}

pub(crate) fn virtual_geometry_node_cluster_cull_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &NODE_CLUSTER_CULL_CONTRACT)?;
    gpu::execute_node_cluster_cull(context.require_gpu()?)
}

pub(crate) fn virtual_geometry_page_feedback_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &PAGE_FEEDBACK_CONTRACT)?;
    gpu::execute_page_feedback(context.require_gpu()?)
}

pub(crate) fn virtual_geometry_visbuffer_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &VISBUFFER_CONTRACT)?;
    gpu::execute_visbuffer(context.require_gpu()?)
}

pub(crate) fn virtual_geometry_debug_overlay_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &DEBUG_OVERLAY_CONTRACT)?;
    gpu::execute_debug_overlay(context.require_gpu()?)
}

pub(crate) fn virtual_geometry_indirect_args_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    match context.pass_name.as_str() {
        "virtual-geometry-indirect-args" => {
            validate_context(context, &INDIRECT_ARGS_CONTRACT)?;
            indirect::execute(context.require_gpu()?)
        }
        "virtual-geometry-terminal-execution" => {
            validate_context(context, &TERMINAL_CONTRACT)?;
            indirect::complete_main(context.require_gpu()?)
        }
        _ => Err(format!(
            "virtual geometry indirect executor rejected unknown pass {}",
            context.pass_name
        )),
    }
}

fn validate_context(
    context: &RenderPassExecutionContext<'_>,
    contract: &RenderPassExecutorContract,
) -> Result<(), String> {
    if context.executor_id.as_str() != contract.executor_id {
        return Err(format!(
            "virtual geometry executor contract mismatch: pass `{}` expected executor `{}`, got `{}`",
            context.pass_name, contract.executor_id, context.executor_id
        ));
    }
    if context.pass_name != contract.pass_name {
        return Err(format!(
            "virtual geometry executor `{}` received pass `{}`, expected `{}`",
            contract.executor_id, context.pass_name, contract.pass_name
        ));
    }
    if context.declared_queue != contract.declared_queue {
        return Err(format!(
            "virtual geometry executor `{}` declared queue mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.declared_queue
        ));
    }
    if !queue_is_compatible(context.queue, contract.declared_queue) {
        return Err(format!(
            "virtual geometry executor `{}` ran on incompatible queue for pass `{}`: declared `{:?}`, actual `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.queue
        ));
    }
    if context.flags != contract.flags {
        return Err(format!(
            "virtual geometry executor `{}` pass flag mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.flags, context.flags
        ));
    }

    let expected = expected_resource_descriptions(contract.resources);
    let actual = actual_resource_descriptions(&context.resources);
    if expected != actual {
        return Err(format!(
            "virtual geometry executor `{}` resource contract mismatch for pass `{}`: expected {:?}, got {:?}",
            contract.executor_id, context.pass_name, expected, actual
        ));
    }

    Ok(())
}

#[cfg(test)]
pub(crate) struct NodeClusterCullReadback {
    collector: Arc<gpu::VisibleClusterReadbackCollector>,
    latest_key: Arc<Mutex<Option<gpu::VisibleClusterReadbackKey>>>,
}

#[cfg(test)]
pub(crate) struct IndirectGpuReadback {
    collector: Arc<indirect::IndirectReadbackCollector>,
    latest_frame_id: Arc<Mutex<Option<u64>>>,
}

#[cfg(test)]
impl IndirectGpuReadback {
    pub(crate) fn new() -> Self {
        Self {
            collector: Arc::new(indirect::IndirectReadbackCollector::default()),
            latest_frame_id: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn executor_registration(&self) -> RenderPassExecutorRegistration {
        let executor = Arc::new(IndirectReadbackExecutor {
            collector: Arc::clone(&self.collector),
            latest_frame_id: Arc::clone(&self.latest_frame_id),
            next_frame_id: AtomicU64::new(1),
        });
        RenderPassExecutorRegistration::new_executor(INDIRECT_ARGS_CONTRACT.executor_id, executor)
    }

    pub(crate) fn read_latest_words(&self) -> Result<indirect::IndirectReadbackWords, String> {
        let frame_id = self
            .latest_frame_id
            .lock()
            .map_err(|_| "virtual geometry indirect readback key lock poisoned".to_owned())?
            .take()
            .ok_or_else(|| {
                "indirect GPU pass did not submit a readback for this frame".to_owned()
            })?;
        self.collector.read_words(frame_id)
    }
}

#[cfg(test)]
impl NodeClusterCullReadback {
    pub(crate) fn new() -> Self {
        Self {
            collector: Arc::new(gpu::VisibleClusterReadbackCollector::default()),
            latest_key: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn executor_registration(&self) -> RenderPassExecutorRegistration {
        let executor = Arc::new(NodeClusterCullReadbackExecutor {
            collector: Arc::clone(&self.collector),
            latest_key: Arc::clone(&self.latest_key),
            next_frame_id: AtomicU64::new(1),
        });
        RenderPassExecutorRegistration::new_executor(
            NODE_CLUSTER_CULL_CONTRACT.executor_id,
            executor,
        )
    }

    pub(crate) fn read_latest_words(&self) -> Result<Vec<u32>, String> {
        self.read_latest_outputs()
            .map(|outputs| outputs.visible_clusters)
    }

    pub(crate) fn read_latest_outputs(&self) -> Result<gpu::CullGpuReadbackWords, String> {
        let key = self
            .latest_key
            .lock()
            .map_err(|_| "visible cluster readback key lock poisoned".to_owned())?
            .take()
            .ok_or_else(|| {
                "node cluster cull did not submit a readback for this frame".to_owned()
            })?;
        self.collector.read_outputs(&key)
    }
}

#[cfg(test)]
struct NodeClusterCullReadbackExecutor {
    collector: Arc<gpu::VisibleClusterReadbackCollector>,
    latest_key: Arc<Mutex<Option<gpu::VisibleClusterReadbackKey>>>,
    next_frame_id: AtomicU64,
}

#[cfg(test)]
impl RenderPassExecutor for NodeClusterCullReadbackExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context, &NODE_CLUSTER_CULL_CONTRACT)?;
        let gpu = context.require_gpu()?;
        gpu::execute_node_cluster_cull(gpu)?;
        let frame_id = self.next_frame_id.fetch_add(1, Ordering::Relaxed);
        let key = gpu::record_visible_cluster_readback(gpu, &self.collector, frame_id)?;
        *self
            .latest_key
            .lock()
            .map_err(|_| "visible cluster readback key lock poisoned".to_owned())? = Some(key);
        Ok(())
    }
}

#[cfg(test)]
struct IndirectReadbackExecutor {
    collector: Arc<indirect::IndirectReadbackCollector>,
    latest_frame_id: Arc<Mutex<Option<u64>>>,
    next_frame_id: AtomicU64,
}

#[cfg(test)]
impl RenderPassExecutor for IndirectReadbackExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        if context.pass_name == "virtual-geometry-terminal-execution" {
            return virtual_geometry_indirect_args_executor(context);
        }
        validate_context(context, &INDIRECT_ARGS_CONTRACT)?;
        let gpu = context.require_gpu()?;
        indirect::execute(gpu)?;
        let frame_id = self.next_frame_id.fetch_add(1, Ordering::Relaxed);
        indirect::record_readback(gpu, &self.collector, frame_id)?;
        *self
            .latest_frame_id
            .lock()
            .map_err(|_| "virtual geometry indirect readback key lock poisoned".to_owned())? =
            Some(frame_id);
        Ok(())
    }
}

fn queue_is_compatible(actual: QueueLane, declared: QueueLane) -> bool {
    actual == declared || (declared != QueueLane::Graphics && actual == QueueLane::Graphics)
}

fn expected_resource_descriptions(resources: &[ExpectedResource]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| resource.description())
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

fn actual_resource_descriptions(resources: &[RenderGraphPassResourceAccess]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| describe_resource(&resource.name, resource.kind, resource.access))
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

fn describe_resource(
    name: &str,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    format!("{access:?}:{kind:?}:{name}")
}

#[cfg(test)]
#[path = "tests/executors.rs"]
mod tests;

pub(crate) use crate::virtual_geometry::materialize_prepared_cull;
