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
fn virtual_geometry_plugin_registrations_require_gpu_recording_context() {
    for registration in crate::render_pass_executor_registrations() {
        let contract = CONTRACTS
            .iter()
            .find(|contract| contract.executor_id == registration.executor_id().as_str())
            .expect("registration should map to an executor contract");
        let mut context = context_for_contract(contract);
        let error = registration
            .execute(&mut context)
            .expect_err("production executor must reject metadata-only execution");
        assert!(error.contains("requires renderer GPU context"), "{error}");
    }
}

#[test]
fn virtual_geometry_async_executors_accept_graphics_queue_fallback() {
    for contract in [NODE_CLUSTER_CULL_CONTRACT, PAGE_FEEDBACK_CONTRACT] {
        let mut context = context_for_contract(&contract);
        context.queue = QueueLane::Graphics;

        validate_context(&context, &contract)
            .unwrap_or_else(|error| panic!("{} fallback failed: {error}", contract.executor_id));
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
                attachment_ops: None,
            })
            .collect(),
    )
}
