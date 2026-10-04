use std::any::Any;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use zircon_editor::core::editing::engine::{
    CommandEffect, EditCommand, EditCommandError, EditContext, EditWorldRoute,
    EditorTransactionEngine, HistoryContextId, MergeMode, SelectionSnapshot,
};
use zircon_editor::core::gateway::{
    EditorRuntimeFrameDemand, EditorRuntimeGateway, EditorRuntimeGatewayHandle,
    EditorRuntimeOperationRoute, GatewayError, GatewaySessionIdentity, RuntimeCapabilities,
    SessionGateway,
};
use zircon_editor::core::play::WorldDomain;
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshBakeRequest, NavigationGeneratedBakeChange,
    NavigationGeneratedBakeSnapshot, NAVIGATION_BAKE_SCENE_OPERATION,
    NAVIGATION_BAKE_SURFACE_OPERATION, NAVIGATION_CLEAR_SURFACE_OPERATION,
    NAVIGATION_RESTORE_BAKE_OPERATION,
};
use zircon_runtime::dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v8};
use zircon_runtime_interface::{
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationPhase,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimeSessionHandle, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use crate::operation_command::NavigationOperationCommand;

fn operation_invocation(
    operation: &str,
    arguments: serde_json::Value,
) -> zircon_editor::core::editor_operation::EditorOperationInvocation {
    zircon_editor::core::editor_operation::EditorOperationInvocation::new(
        zircon_editor::core::editor_operation::EditorOperationPath::parse(operation).unwrap(),
    )
    .with_arguments(arguments)
}

fn factory_request_payload(operation: &str, arguments: serde_json::Value) -> serde_json::Value {
    let registration = crate::plugin_registration();
    let operation_path =
        zircon_editor::core::editor_operation::EditorOperationPath::parse(operation).unwrap();
    let command = registration
        .extensions
        .operation_factory(&operation_path)
        .expect("navigation operation factory")
        .create(&operation_invocation(operation, arguments))
        .expect("navigation operation command");
    command
        .command()
        .journal_payload()
        .expect("navigation command journal payload")
        .payload()
        .get("request")
        .cloned()
        .and_then(|request| request.get("payload").cloned())
        .expect("navigation runtime request payload")
}

#[test]
fn navigation_selected_surface_factory_accepts_typed_object_arguments() {
    assert_eq!(
        factory_request_payload(
            NAVIGATION_BAKE_SURFACE_OPERATION,
            serde_json::json!({ "surface_entity": 41, "force_full_rebuild": true })
        ),
        serde_json::json!({
            "surface_entity": 41,
            "agent_type": null,
            "output_asset": null,
            "force_full_rebuild": true
        })
    );
    assert_eq!(
        factory_request_payload(
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            serde_json::json!({ "surface_entity": 73 })
        ),
        serde_json::json!({ "surface_entity": 73 })
    );
}

#[test]
fn navigation_selected_surface_factory_rejects_missing_or_malformed_selection_arguments() {
    let registration = crate::plugin_registration();
    let operation_path = zircon_editor::core::editor_operation::EditorOperationPath::parse(
        NAVIGATION_BAKE_SURFACE_OPERATION,
    )
    .unwrap();
    let factory = registration
        .extensions
        .operation_factory(&operation_path)
        .expect("navigation bake surface factory");

    for arguments in [
        serde_json::Value::Null,
        serde_json::json!([]),
        serde_json::json!(["not-an-entity", false]),
        serde_json::json!([41]),
        serde_json::json!([41, false, "extra"]),
        serde_json::json!({ "surface_entity": "not-an-entity", "force_full_rebuild": false }),
        serde_json::json!({ "force_full_rebuild": false }),
    ] {
        assert!(factory
            .create(&operation_invocation(
                NAVIGATION_BAKE_SURFACE_OPERATION,
                arguments
            ))
            .is_err());
    }
}

struct RecordingGateway {
    state: Mutex<RecordingGatewayState>,
}

struct RecordingGatewayState {
    next_handle: u64,
    current: NavigationGeneratedBakeSnapshot,
    requests: Vec<ZrRuntimeOperationSubmitRequestV1>,
    results: BTreeMap<u64, ZrRuntimeOperationResultV1>,
    foreign_progress: bool,
    failed_result: bool,
    wrong_progress_abi: bool,
    wrong_result_abi: bool,
}

impl RecordingGateway {
    fn new() -> Self {
        Self {
            state: Mutex::new(RecordingGatewayState {
                next_handle: 1,
                current: NavigationGeneratedBakeSnapshot::empty(Some(7)),
                requests: Vec::new(),
                results: BTreeMap::new(),
                foreign_progress: false,
                failed_result: false,
                wrong_progress_abi: false,
                wrong_result_abi: false,
            }),
        }
    }

    fn with_foreign_progress() -> Self {
        let gateway = Self::new();
        gateway.state.lock().unwrap().foreign_progress = true;
        gateway
    }

    fn with_failed_result() -> Self {
        let gateway = Self::new();
        gateway.state.lock().unwrap().failed_result = true;
        gateway
    }

    fn with_wrong_progress_abi() -> Self {
        let gateway = Self::new();
        gateway.state.lock().unwrap().wrong_progress_abi = true;
        gateway
    }

    fn with_wrong_result_abi() -> Self {
        let gateway = Self::new();
        gateway.state.lock().unwrap().wrong_result_abi = true;
        gateway
    }

    fn operation_ids(&self) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .map(|request| request.operation_id.clone())
            .collect()
    }

    fn current(&self) -> NavigationGeneratedBakeSnapshot {
        self.state.lock().unwrap().current.clone()
    }
}

impl EditorRuntimeGateway for RecordingGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(3)
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        Ok(EditorRuntimeFrameDemand::OnDemand)
    }

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        let mut state = self.state.lock().unwrap();
        let handle = ZrRuntimeOperationHandle::new(state.next_handle);
        state.next_handle += 1;
        let before = state.current.clone();
        let after = if request.operation_id == NAVIGATION_BAKE_SCENE_OPERATION {
            NavigationGeneratedBakeSnapshot {
                surface_entity: Some(7),
                asset: Some(NavMeshAsset::from_triangle_mesh(
                    "humanoid",
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                    vec![0, 1, 2],
                    0,
                )),
                output_asset: Some("res://navigation/generated.znavmesh".to_string()),
            }
        } else if request.operation_id == NAVIGATION_RESTORE_BAKE_OPERATION {
            serde_json::from_value(request.payload.clone()).unwrap()
        } else {
            return Err(GatewayError::Runtime {
                message: format!("unexpected operation {}", request.operation_id),
            });
        };
        state.current = after.clone();
        let change = NavigationGeneratedBakeChange {
            before,
            after,
            report: None,
        };
        let result = if state.failed_result {
            ZrRuntimeOperationResultV1::failed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                handle,
                request.operation_id.clone(),
                "runtime rejected the generated bake",
            )
        } else {
            ZrRuntimeOperationResultV1::succeeded(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                handle,
                request.operation_id.clone(),
                serde_json::to_value(change).unwrap(),
            )
        };
        let mut result = result;
        if state.wrong_result_abi {
            result.abi_version = ZIRCON_RUNTIME_ABI_VERSION_V1 + 1;
        }
        state.results.insert(handle.raw(), result);
        state.requests.push(request);
        Ok(handle)
    }

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        let state = self.state.lock().unwrap();
        let handle = if state.foreign_progress {
            ZrRuntimeOperationHandle::new(handle.raw() + 1)
        } else {
            handle
        };
        let (phase, detail_kind) = if state.failed_result {
            (
                ZrRuntimeOperationPhase::Failed,
                ZrRuntimeOperationDetailKindV2::OwnerApplyFailed,
            )
        } else {
            (
                ZrRuntimeOperationPhase::Completed,
                ZrRuntimeOperationDetailKindV2::None,
            )
        };
        let mut status = ZrRuntimeOperationStatusV2::new(handle, phase, 1, 1, detail_kind, 0);
        if state.wrong_progress_abi {
            status.abi_version = ZIRCON_RUNTIME_ABI_VERSION_V2 + 1;
        }
        Ok(status)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.state
            .lock()
            .unwrap()
            .results
            .remove(&handle.raw())
            .ok_or_else(|| GatewayError::Runtime {
                message: "missing operation result".to_string(),
            })
    }
}

struct GatewayEditContext {
    gateway: EditorRuntimeGatewayHandle,
    active_route: Option<EditWorldRoute>,
}

impl GatewayEditContext {
    fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            gateway,
            active_route: None,
        }
    }
}

impl EditContext for GatewayEditContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        if world_domain != WorldDomain::Edit {
            return Err(EditCommandError::WorldRouteUnavailable { world_domain });
        }
        Ok(EditWorldRoute::runtime(
            world_domain,
            self.gateway.identity(),
        ))
    }

    fn activate_world_route(&mut self, route: &EditWorldRoute) -> Result<(), EditCommandError> {
        if self.capture_world_route(route.world_domain())? != *route {
            return Err(EditCommandError::WorldRouteStale {
                world_domain: route.world_domain(),
            });
        }
        self.active_route = Some(route.clone());
        Ok(())
    }

    fn retire_world_route(&mut self, world_domain: WorldDomain) -> Result<(), EditCommandError> {
        if self
            .active_route
            .as_ref()
            .is_some_and(|route| route.world_domain() == world_domain)
        {
            return Err(EditCommandError::InvariantViolation {
                invariant: "the active fixture world route cannot be retired",
            });
        }
        Ok(())
    }

    fn runtime_operations(&self) -> Result<EditorRuntimeOperationRoute, EditCommandError> {
        let identity = self
            .active_route
            .as_ref()
            .and_then(EditWorldRoute::gateway_identity)
            .cloned()
            .unwrap_or_else(|| self.gateway.identity());
        EditorRuntimeOperationRoute::capture_at_identity(&self.gateway, &identity).map_err(
            |source| EditCommandError::ExternalEffect {
                source: Box::new(source),
            },
        )
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn error_chain_contains(error: &(dyn Error + 'static), needle: &str) -> bool {
    let mut current = Some(error);
    while let Some(source) = current {
        if source.to_string().contains(needle) {
            return true;
        }
        current = source.source();
    }
    false
}

#[test]
fn recording_gateway_failed_result_polls_owner_apply_failure_status() {
    let gateway = RecordingGateway::with_failed_result();
    let handle = gateway
        .submit_operation(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
        ))
        .unwrap();
    let status = gateway.poll_operation(handle).unwrap();
    assert_eq!(status.handle, handle);
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Failed));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::OwnerApplyFailed)
    );
    assert_eq!(
        gateway.harvest_operation(handle).unwrap().failure(),
        Some("runtime rejected the generated bake")
    );
}

#[test]
fn runtime_operation_route_keeps_one_gateway_generation_for_the_whole_chain() {
    let first = Arc::new(RecordingGateway::new());
    let replacement = Arc::new(RecordingGateway::new());
    let gateway = EditorRuntimeGatewayHandle::new(first.clone());
    let route = EditorRuntimeOperationRoute::capture_at_identity(&gateway, &gateway.identity())
        .expect("the initial runtime operation route should capture");
    gateway
        .replace(replacement.clone())
        .expect("the stable gateway handle should accept a replacement");

    let handle = route
        .submit_operation(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
        ))
        .expect("the pinned route should submit through its captured generation");
    let status = route
        .poll_operation(handle)
        .expect("the pinned route should poll the same generation");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Completed));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::None)
    );
    route
        .harvest_operation(handle)
        .expect("the pinned route should harvest the same generation");

    assert!(first.current().asset.is_some());
    assert!(replacement.current().asset.is_none());
}

#[test]
fn navigation_operation_command_undo_and_redo_restore_snapshots_without_rebake() {
    let gateway = Arc::new(RecordingGateway::new());
    let engine = EditorTransactionEngine::new(GatewayEditContext::new(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
    ));
    let command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest {
            force_full_rebuild: true,
            ..NavMeshBakeRequest::default()
        })
        .unwrap(),
    ));

    engine
        .execute_operation(
            "Bake Navigation Scene",
            HistoryContextId::Global,
            None,
            MergeMode::Disable,
            Box::new(command),
        )
        .unwrap();
    assert!(gateway.current().asset.is_some());
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert!(gateway.current().asset.is_none());
    assert!(engine.redo(HistoryContextId::Global).unwrap());
    assert!(gateway.current().asset.is_some());
    assert_eq!(
        gateway.operation_ids(),
        vec![
            NAVIGATION_BAKE_SCENE_OPERATION.to_string(),
            NAVIGATION_RESTORE_BAKE_OPERATION.to_string(),
            NAVIGATION_RESTORE_BAKE_OPERATION.to_string(),
        ]
    );
}

#[test]
fn navigation_operation_command_marks_post_submit_protocol_failure_as_applied() {
    let gateway = Arc::new(RecordingGateway::with_foreign_progress());
    let mut context = GatewayEditContext::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let mut command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
    ));

    let error = command.apply(&mut context).unwrap_err();

    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(error_chain_contains(
        &error,
        "progress returned a foreign operation handle"
    ));
    assert!(gateway.current().asset.is_some());
}

#[test]
fn navigation_operation_command_marks_terminal_runtime_failure_as_applied() {
    let gateway = Arc::new(RecordingGateway::with_failed_result());
    let mut context = GatewayEditContext::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let mut command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
    ));

    let error = command.apply(&mut context).unwrap_err();

    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(error_chain_contains(
        &error,
        "runtime rejected the generated bake"
    ));
    assert!(gateway.current().asset.is_some());
}

#[test]
fn navigation_post_submit_failure_faults_transaction_engine_when_rollback_is_unknown() {
    let gateway = Arc::new(RecordingGateway::with_failed_result());
    let engine = EditorTransactionEngine::new(GatewayEditContext::new(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
    ));
    let command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
    ));

    engine
        .execute_operation(
            "Bake Navigation Scene",
            HistoryContextId::Global,
            None,
            MergeMode::Disable,
            Box::new(command),
        )
        .expect_err("unknown external effect must fail the transaction");

    assert!(gateway.current().asset.is_some());
    assert!(matches!(
        engine.undo(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}

#[test]
fn navigation_operation_command_rejects_progress_with_foreign_abi() {
    let gateway = Arc::new(RecordingGateway::with_wrong_progress_abi());
    let mut context = GatewayEditContext::new(EditorRuntimeGatewayHandle::new(gateway));
    let mut command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
    ));

    let error = command.apply(&mut context).unwrap_err();

    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(error_chain_contains(&error, "progress ABI version"));
}

#[test]
fn navigation_operation_command_rejects_result_with_foreign_abi() {
    let gateway = Arc::new(RecordingGateway::with_wrong_result_abi());
    let mut context = GatewayEditContext::new(EditorRuntimeGatewayHandle::new(gateway));
    let mut command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_BAKE_SCENE_OPERATION,
        serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
    ));

    let error = command.apply(&mut context).unwrap_err();

    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(error_chain_contains(&error, "result ABI version"));
}

#[test]
#[ignore = "managed real DynamicSession operation consumer; run alone with --test-threads=1"]
fn navigation_editor_command_consumes_real_dynamic_session_tick_poll_harvest_and_reload_boundary() {
    let api = unsafe { zircon_runtime_get_api_v8(std::ptr::null()) };
    assert!(
        !api.is_null(),
        "linked runtime API table should be available"
    );
    let api = unsafe { std::ptr::read(api) };
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![zircon_plugin_navigation_runtime::plugin_registration()],
    )
    .expect("the linked DynamicSession should admit the navigation runtime registration");
    let gateway = Arc::new(unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            session,
            GatewaySessionIdentity::new(1, session, 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(Default::default()),
        )
        .expect("the editor gateway should retain the real DynamicSession identity");
    });
    let gateway_handle = EditorRuntimeGatewayHandle::new(gateway.clone());
    let engine = EditorTransactionEngine::new(GatewayEditContext::new(gateway_handle.clone()));
    let command = NavigationOperationCommand::new(ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        NAVIGATION_CLEAR_SURFACE_OPERATION,
        serde_json::json!({ "surface_entity": 7 }),
    ));

    engine
        .execute_operation(
            "Clear Navigation Surface Bake",
            HistoryContextId::Global,
            None,
            MergeMode::Disable,
            Box::new(command),
        )
        .expect("the normal editor command must drive DynamicSession through tick/poll/harvest");
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert!(engine.redo(HistoryContextId::Global).unwrap());
    let _ = gateway_handle
        .drain_world_invalidations()
        .expect("normal editor consumer must be able to sample runtime mutation invalidations");

    let destroy = api
        .destroy_session
        .expect("linked runtime API exposes session destruction");
    let status = unsafe { destroy(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok);
}
