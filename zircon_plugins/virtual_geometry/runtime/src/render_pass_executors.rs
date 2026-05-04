use zircon_runtime::graphics::RenderPassExecutionContext;
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

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
    validate_context(context, &PREPARE_CONTRACT)
}

pub(crate) fn virtual_geometry_node_cluster_cull_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &NODE_CLUSTER_CULL_CONTRACT)
}

pub(crate) fn virtual_geometry_page_feedback_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &PAGE_FEEDBACK_CONTRACT)
}

pub(crate) fn virtual_geometry_visbuffer_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &VISBUFFER_CONTRACT)
}

pub(crate) fn virtual_geometry_debug_overlay_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &DEBUG_OVERLAY_CONTRACT)
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
mod tests {
    use zircon_runtime::graphics::RenderPassExecutorId;

    use super::*;

    const CONTRACTS: &[RenderPassExecutorContract] = &[
        PREPARE_CONTRACT,
        NODE_CLUSTER_CULL_CONTRACT,
        PAGE_FEEDBACK_CONTRACT,
        VISBUFFER_CONTRACT,
        DEBUG_OVERLAY_CONTRACT,
    ];

    #[test]
    fn virtual_geometry_executors_accept_declared_feature_pass_contexts() {
        for contract in CONTRACTS {
            validate_context(&context_for_contract(contract), contract)
                .unwrap_or_else(|error| panic!("{} failed: {error}", contract.executor_id));
        }
    }

    #[test]
    fn virtual_geometry_plugin_registrations_execute_contract_bound_passes() {
        for registration in crate::render_pass_executor_registrations() {
            let contract = CONTRACTS
                .iter()
                .find(|contract| contract.executor_id == registration.executor_id().as_str())
                .expect("registration should map to an executor contract");
            let mut context = context_for_contract(contract);
            registration
                .execute(&mut context)
                .unwrap_or_else(|error| panic!("{} failed: {error}", contract.executor_id));
        }
    }

    #[test]
    fn virtual_geometry_async_executors_accept_graphics_queue_fallback() {
        for contract in [NODE_CLUSTER_CULL_CONTRACT, PAGE_FEEDBACK_CONTRACT] {
            let mut context = context_for_contract(&contract);
            context.queue = QueueLane::Graphics;

            validate_context(&context, &contract).unwrap_or_else(|error| {
                panic!("{} fallback failed: {error}", contract.executor_id)
            });
        }
    }

    #[test]
    fn virtual_geometry_executor_rejects_legacy_runtime_pass_name() {
        let mut context = context_for_contract(&PREPARE_CONTRACT);
        context.pass_name = "runtime-virtual-geometry-prepare".to_string();

        let error = virtual_geometry_prepare_executor(&mut context).unwrap_err();

        assert!(
            error.contains("expected `virtual-geometry-prepare`"),
            "{error}"
        );
    }

    #[test]
    fn virtual_geometry_executor_rejects_resource_contract_drift() {
        let mut context = context_for_contract(&VISBUFFER_CONTRACT);
        context.resources.pop();

        let error = virtual_geometry_visbuffer_executor(&mut context).unwrap_err();

        assert!(error.contains("resource contract mismatch"), "{error}");
        assert!(error.contains("scene-depth"), "{error}");
    }

    #[test]
    fn virtual_geometry_executor_rejects_unexpected_queue_drift() {
        let mut context = context_for_contract(&PREPARE_CONTRACT);
        context.queue = QueueLane::AsyncCompute;

        let error = virtual_geometry_prepare_executor(&mut context).unwrap_err();

        assert!(error.contains("incompatible queue"), "{error}");
    }

    fn context_for_contract(
        contract: &RenderPassExecutorContract,
    ) -> RenderPassExecutionContext<'static> {
        RenderPassExecutionContext::with_declared_graph_metadata_and_resources(
            contract.pass_name,
            RenderPassExecutorId::new(contract.executor_id),
            contract.declared_queue,
            contract.declared_queue,
            contract.flags,
            contract
                .resources
                .iter()
                .map(|resource| RenderGraphPassResourceAccess {
                    name: resource.name.to_string(),
                    kind: resource.kind,
                    access: resource.access,
                })
                .collect(),
        )
    }
}
