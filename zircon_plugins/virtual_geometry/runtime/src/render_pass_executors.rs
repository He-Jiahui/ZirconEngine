use zircon_runtime::graphics::RenderPassExecutionContext;
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

mod gpu;

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
];

const PAGE_FEEDBACK_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "virtual-geometry-visible-clusters",
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
#[path = "tests/render_pass_executors.rs"]
mod tests;
