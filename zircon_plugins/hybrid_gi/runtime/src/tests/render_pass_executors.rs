use zircon_runtime::graphics::RenderPassExecutorId;

use super::*;

const CONTRACTS: &[RenderPassExecutorContract] = &[
    SCENE_PREPARE_CONTRACT,
    TRACE_SCHEDULE_CONTRACT,
    RESOLVE_CONTRACT,
];

#[test]
fn hybrid_gi_executors_accept_declared_feature_pass_contexts() {
    for contract in CONTRACTS {
        validate_context(&context_for_contract(contract), contract)
            .unwrap_or_else(|error| panic!("{} failed: {error}", contract.executor_id));
    }
}

#[test]
fn hybrid_gi_plugin_registrations_execute_contract_bound_passes() {
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
fn hybrid_gi_async_executor_accepts_graphics_queue_fallback() {
    let mut context = context_for_contract(&TRACE_SCHEDULE_CONTRACT);
    context.queue = QueueLane::Graphics;

    hybrid_gi_trace_schedule_executor(&mut context)
        .unwrap_or_else(|error| panic!("trace schedule fallback failed: {error}"));
}

#[test]
fn hybrid_gi_read_only_scene_depth_accepts_external_or_transient_graph_resources() {
    let mut scene_prepare = context_for_contract(&SCENE_PREPARE_CONTRACT);
    scene_prepare.resources[0].kind = RenderGraphResourceKind::TransientTexture;
    hybrid_gi_scene_prepare_executor(&mut scene_prepare)
        .unwrap_or_else(|error| panic!("scene-depth transient texture failed: {error}"));
}

#[test]
fn hybrid_gi_resolve_accepts_external_or_transient_scene_velocity() {
    for kind in [
        RenderGraphResourceKind::External,
        RenderGraphResourceKind::TransientTexture,
    ] {
        let mut resolve = context_for_contract(&RESOLVE_CONTRACT);
        resolve
            .resources
            .iter_mut()
            .find(|resource| resource.name == SCENE_VELOCITY_RESOURCE)
            .expect("resolve scene-velocity resource")
            .kind = kind;

        hybrid_gi_resolve_executor(&mut resolve)
            .unwrap_or_else(|error| panic!("{kind:?} scene-velocity should be accepted: {error}"));
    }
}

#[test]
fn hybrid_gi_executor_rejects_legacy_runtime_pass_name() {
    let mut context = context_for_contract(&SCENE_PREPARE_CONTRACT);
    context.pass_name = "runtime-hybrid-gi-scene-prepare".to_string();

    let error = hybrid_gi_scene_prepare_executor(&mut context).unwrap_err();

    assert!(
        error.contains("expected `hybrid-gi-scene-prepare`"),
        "{error}"
    );
}

#[test]
fn hybrid_gi_executor_rejects_resource_contract_drift() {
    let mut context = context_for_contract(&SCENE_PREPARE_CONTRACT);
    context.resources[0].kind = RenderGraphResourceKind::TransientBuffer;

    let error = hybrid_gi_scene_prepare_executor(&mut context).unwrap_err();

    assert!(error.contains("resource contract mismatch"), "{error}");
    assert!(error.contains("scene-depth"), "{error}");
}

#[test]
fn hybrid_gi_executor_rejects_unexpected_queue_drift() {
    let mut context = context_for_contract(&RESOLVE_CONTRACT);
    context.queue = QueueLane::AsyncCopy;

    let error = hybrid_gi_resolve_executor(&mut context).unwrap_err();

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
                kind: match resource.kind {
                    ExpectedResourceKind::Exact(kind) => kind,
                    ExpectedResourceKind::AnyOf(kinds) => kinds[0],
                },
                access: resource.access,
                attachment_ops: None,
            })
            .collect(),
    )
}
