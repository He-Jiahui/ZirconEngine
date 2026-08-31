use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrRuntimeEventV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use crate::core::gateway::{
    EditorRuntimeFrame, EditorRuntimeGateway, EditorRuntimeGatewayHandle,
    EditorRuntimePluginEventPage, GatewayError,
};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerApplyError, EditorRuntimeEventConsumerError,
    EditorRuntimeEventConsumerHost, EditorRuntimeEventConsumerManifest,
    EditorRuntimeEventConsumerRegistration, EditorRuntimeEventConsumerRegistry,
    EditorRuntimeEventConsumerState,
};

const CONSUMER_ID: &str = "tests.consumer";
const EVENT_ID: &str = "tests.events.tick";
const SCHEMA: &str = "tests.events.tick.v1";
const CAPABILITY: &str = "editor.tests.consumer";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TestPayload {
    value: u32,
}

#[derive(Debug, thiserror::Error)]
enum TestConsumerError {
    #[error("wrong state session")]
    WrongSession,
}

#[derive(Default)]
struct TestState {
    session: Option<u64>,
    deliveries: Vec<(u64, TestPayload)>,
}

impl EditorRuntimeEventConsumerState for TestState {
    type Payload = TestPayload;
    type Error = TestConsumerError;

    fn begin_session(&mut self, play_session_id: u64) {
        self.session = Some(play_session_id);
        self.deliveries.clear();
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        if self.session != Some(play_session_id) {
            return Err(TestConsumerError::WrongSession);
        }
        self.deliveries.push((sequence, payload));
        Ok(())
    }

    fn end_session(&mut self, play_session_id: u64) {
        if self.session == Some(play_session_id) {
            *self = Self::default();
        }
    }
}

struct FakeRuntimeGateway {
    session: ZrRuntimeSessionHandle,
    identity: GatewaySessionIdentity,
    deliveries: Mutex<BTreeMap<u64, Vec<ZrRuntimePluginEventDeliveryV1>>>,
    unsubscribed: Mutex<Vec<u64>>,
    unsubscribe_removed: Mutex<bool>,
    subscribe_calls: AtomicU64,
    drain_calls: AtomicU64,
    fail_subscribe_call: Mutex<Option<u64>>,
}

impl FakeRuntimeGateway {
    fn new(session: u64) -> Self {
        Self::with_identity(GatewaySessionIdentity::new(
            1,
            ZrRuntimeSessionHandle::new(session),
            1,
            None,
        ))
    }

    fn with_identity(identity: GatewaySessionIdentity) -> Self {
        Self {
            session: identity.runtime_session(),
            identity,
            deliveries: Mutex::new(BTreeMap::new()),
            unsubscribed: Mutex::new(Vec::new()),
            unsubscribe_removed: Mutex::new(true),
            subscribe_calls: AtomicU64::new(0),
            drain_calls: AtomicU64::new(0),
            fail_subscribe_call: Mutex::new(None),
        }
    }

    fn fail_subscribe_on(&self, call: u64) {
        *self.fail_subscribe_call.lock().unwrap() = Some(call);
    }

    fn push(&self, delivery: ZrRuntimePluginEventDeliveryV1) {
        self.deliveries
            .lock()
            .unwrap()
            .entry(delivery.subscription.raw())
            .or_default()
            .push(delivery);
    }
}

impl EditorRuntimeGateway for FakeRuntimeGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        self.identity.clone()
    }

    fn handle_event(&self, _event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        Ok(())
    }

    fn capture_frame(
        &self,
        _viewport: ZrRuntimeViewportHandle,
        _size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        Ok(EditorRuntimeFrame::empty(1))
    }

    fn subscribe_plugin_event(
        &self,
        _event_id: &str,
        _payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        let call = self.subscribe_calls.fetch_add(1, Ordering::Relaxed) + 1;
        if *self.fail_subscribe_call.lock().unwrap() == Some(call) {
            return Err(GatewayError::CapabilityMissing {
                capability: "runtime.plugin_event.subscribe",
            });
        }
        Ok(Some(ZrRuntimePluginEventSubscriptionHandle::new(10 + call)))
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.unsubscribed.lock().unwrap().push(subscription.raw());
        Ok(*self.unsubscribe_removed.lock().unwrap())
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        self.drain_calls.fetch_add(1, Ordering::Relaxed);
        Ok(EditorRuntimePluginEventPage::synthetic(
            self.deliveries
                .lock()
                .unwrap()
                .remove(&subscription.raw())
                .unwrap_or_default(),
        ))
    }

    fn submit_operation(
        &self,
        _request: zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

#[test]
fn replaced_runtime_cannot_receive_an_old_consumer_subscription() {
    let state = Arc::new(Mutex::new(TestState::default()));
    let manifest = EditorRuntimeEventConsumerManifest::new(CONSUMER_ID, EVENT_ID, SCHEMA)
        .with_required_capability(CAPABILITY);
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry
        .register(EditorRuntimeEventConsumerRegistration::typed(
            manifest,
            state.clone(),
        ))
        .unwrap();
    let first = Arc::new(FakeRuntimeGateway::new(7));
    let gateway = EditorRuntimeGatewayHandle::new(first.clone());
    let host = EditorRuntimeEventConsumerHost::new(gateway.clone());
    host.register(registry).unwrap();
    host.begin_play_session(1, &[CAPABILITY.to_string()])
        .unwrap();

    let replacement = Arc::new(FakeRuntimeGateway::new(8));
    replacement.push(delivery(8, 1, SCHEMA));
    gateway.replace(replacement.clone()).unwrap();

    let report = host.pump().expect("a stale consumer is retired locally");
    assert_eq!(report.applied(), 0);
    assert_eq!(report.stale_consumers(), 1);
    assert_eq!(replacement.drain_calls.load(Ordering::Relaxed), 0);
    assert!(host.active_consumer_count() == 0);
}

#[test]
fn replacement_with_reused_raw_session_and_subscription_still_retires_the_old_consumer() {
    let state = Arc::new(Mutex::new(TestState::default()));
    let manifest = EditorRuntimeEventConsumerManifest::new(CONSUMER_ID, EVENT_ID, SCHEMA)
        .with_required_capability(CAPABILITY);
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry
        .register(EditorRuntimeEventConsumerRegistration::typed(
            manifest, state,
        ))
        .unwrap();

    let raw_session = ZrRuntimeSessionHandle::new(7);
    let first = Arc::new(FakeRuntimeGateway::with_identity(
        GatewaySessionIdentity::new(31, raw_session, 41, Some(Arc::from("E:/Projects/First"))),
    ));
    let gateway = EditorRuntimeGatewayHandle::new(first);
    let host = EditorRuntimeEventConsumerHost::new(gateway.clone());
    host.register(registry).unwrap();
    host.begin_play_session(1, &[CAPABILITY.to_string()])
        .unwrap();

    let replacement = Arc::new(FakeRuntimeGateway::with_identity(
        GatewaySessionIdentity::new(31, raw_session, 42, Some(Arc::from("E:/Projects/Second"))),
    ));
    replacement.push(delivery(7, 1, SCHEMA));
    gateway.replace(replacement.clone()).unwrap();

    let report = host
        .pump()
        .expect("the old consumer must retire without draining the replacement transport");
    assert_eq!(report.applied(), 0);
    assert_eq!(report.stale_consumers(), 1);
    assert_eq!(replacement.drain_calls.load(Ordering::Relaxed), 0);
    assert_eq!(host.active_consumer_count(), 0);
}

fn test_host() -> (
    EditorRuntimeEventConsumerHost,
    Arc<FakeRuntimeGateway>,
    Arc<Mutex<TestState>>,
) {
    let state = Arc::new(Mutex::new(TestState::default()));
    let manifest = EditorRuntimeEventConsumerManifest::new(CONSUMER_ID, EVENT_ID, SCHEMA)
        .with_required_capability(CAPABILITY);
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry
        .register(EditorRuntimeEventConsumerRegistration::typed(
            manifest,
            state.clone(),
        ))
        .unwrap();
    let client = Arc::new(FakeRuntimeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(client.clone()));
    host.register(registry).unwrap();
    (host, client, state)
}

fn delivery(session: u64, sequence: u64, schema: &str) -> ZrRuntimePluginEventDeliveryV1 {
    ZrRuntimePluginEventDeliveryV1::new(
        session,
        ZrRuntimePluginEventSubscriptionHandle::new(11),
        EVENT_ID,
        schema,
        sequence,
        serde_json::json!({"value": sequence as u32}),
    )
}

#[test]
fn consumer_host_binds_only_with_capability_and_clears_state_on_pie_exit() {
    let (host, client, state) = test_host();
    host.begin_play_session(100, &[]).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    host.end_play_session(100).unwrap();

    host.begin_play_session(101, &[CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(host.active_consumer_count(), 1);
    client.push(delivery(7, 1, SCHEMA));
    assert_eq!(host.pump().unwrap(), 1);
    assert_eq!(state.lock().unwrap().deliveries[0].0, 1);
    assert_eq!(state.lock().unwrap().session, Some(101));

    host.end_play_session(101).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert!(state.lock().unwrap().session.is_none());
    assert_eq!(*client.unsubscribed.lock().unwrap(), [11]);
}

#[test]
fn consumer_host_rejects_cross_session_wrong_schema_and_stale_sequence() {
    let (host, client, _) = test_host();
    host.begin_play_session(200, &[CAPABILITY.to_string()])
        .unwrap();
    client.push(delivery(8, 1, SCHEMA));
    assert!(matches!(
        host.pump(),
        Err(EditorRuntimeEventConsumerError::WrongSession { .. })
    ));

    client.push(delivery(7, 1, "wrong.schema"));
    assert!(matches!(
        host.pump(),
        Err(EditorRuntimeEventConsumerError::SchemaMismatch { .. })
    ));

    client.push(delivery(7, 2, SCHEMA));
    host.pump().unwrap();
    client.push(delivery(7, 2, SCHEMA));
    assert!(matches!(
        host.pump(),
        Err(EditorRuntimeEventConsumerError::StaleSequence { .. })
    ));
    host.end_play_session(200).unwrap();
}

#[test]
fn consumer_host_parses_raw_payload_at_the_typed_consumer_boundary() {
    let (host, client, state) = test_host();
    host.begin_play_session(250, &[CAPABILITY.to_string()])
        .unwrap();

    let raw_delivery: ZrRuntimePluginEventDeliveryV1 = serde_json::from_str(
        r#"{
            "playSessionId": 7,
            "subscription": 11,
            "eventId": "tests.events.tick",
            "payloadSchema": "tests.events.tick.v1",
            "sequence": 1,
            "payload": { "value": 12 }
        }"#,
    )
    .unwrap();
    assert_eq!(raw_delivery.payload.get(), r#"{ "value": 12 }"#);
    client.push(raw_delivery);
    assert_eq!(host.pump().unwrap(), 1);
    assert_eq!(
        state.lock().unwrap().deliveries,
        vec![(1, TestPayload { value: 12 })]
    );

    let wrong_shape: ZrRuntimePluginEventDeliveryV1 = serde_json::from_str(
        r#"{
            "playSessionId": 7,
            "subscription": 11,
            "eventId": "tests.events.tick",
            "payloadSchema": "tests.events.tick.v1",
            "sequence": 2,
            "payload": { "value": "not a number" }
        }"#,
    )
    .unwrap();
    client.push(wrong_shape);
    assert!(matches!(
        host.pump(),
        Err(EditorRuntimeEventConsumerError::Payload {
            source: EditorRuntimeEventConsumerApplyError::Decode { .. },
            ..
        })
    ));
    host.end_play_session(250).unwrap();
}

#[test]
fn consumer_host_reconciles_capability_disable_after_an_event() {
    let (host, client, state) = test_host();
    host.begin_play_session(300, &[CAPABILITY.to_string()])
        .unwrap();
    client.push(delivery(7, 1, SCHEMA));
    assert_eq!(host.pump().unwrap(), 1);

    host.reconcile_enabled_capabilities(&[]).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(*client.unsubscribed.lock().unwrap(), [11]);
    assert!(state.lock().unwrap().session.is_none());

    client.push(delivery(7, 2, SCHEMA));
    assert_eq!(host.pump().unwrap(), 0);
    assert!(state.lock().unwrap().deliveries.is_empty());
    host.end_play_session(300).unwrap();
}

#[test]
fn consumer_host_locally_retires_when_runtime_refuses_subscription_removal() {
    let (host, client, state) = test_host();
    host.begin_play_session(400, &[CAPABILITY.to_string()])
        .unwrap();
    *client.unsubscribe_removed.lock().unwrap() = false;

    assert!(matches!(
        host.reconcile_enabled_capabilities(&[]),
        Err(EditorRuntimeEventConsumerError::Gateway { .. })
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert!(state.lock().unwrap().session.is_none());
    assert_eq!(*client.unsubscribed.lock().unwrap(), [11]);
    host.end_play_session(400).unwrap();
}

#[test]
fn consumer_host_locally_retires_failed_pie_exit_subscriptions() {
    let (host, client, state) = test_host();
    host.begin_play_session(500, &[CAPABILITY.to_string()])
        .unwrap();
    *client.unsubscribe_removed.lock().unwrap() = false;

    assert!(matches!(
        host.end_play_session(500),
        Err(EditorRuntimeEventConsumerError::Gateway { .. })
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert!(state.lock().unwrap().session.is_none());
    assert_eq!(*client.unsubscribed.lock().unwrap(), [11]);
}

#[test]
fn consumer_host_reports_begin_rollback_failure_after_local_retirement() {
    let client = Arc::new(FakeRuntimeGateway::new(7));
    client.fail_subscribe_on(2);
    *client.unsubscribe_removed.lock().unwrap() = false;
    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(client.clone()));
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    for (consumer_id, event_id) in [
        ("tests.consumer.a", "tests.events.a"),
        ("tests.consumer.b", "tests.events.b"),
    ] {
        registry
            .register(EditorRuntimeEventConsumerRegistration::typed(
                EditorRuntimeEventConsumerManifest::new(consumer_id, event_id, SCHEMA)
                    .with_required_capability(CAPABILITY),
                Arc::new(Mutex::new(TestState::default())),
            ))
            .unwrap();
    }
    host.register(registry).unwrap();

    assert!(matches!(
        host.begin_play_session(600, &[CAPABILITY.to_string()]),
        Err(EditorRuntimeEventConsumerError::Cleanup { .. })
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert_eq!(*client.unsubscribed.lock().unwrap(), [11]);
}

#[test]
fn menu_action_completes_local_play_retirement_after_remote_consumer_cleanup_failure() {
    let source = include_str!("../ui/host/editor_event_execution/menu_action.rs");
    let enter = source
        .split("MenuAction::EnterPlayMode =>")
        .nth(1)
        .and_then(|body| body.split("MenuAction::ExitPlayMode =>").next())
        .expect("enter play mode action body");
    let pending = enter
        .find("runtime_event_consumer_session_active()")
        .expect("enter failure should inspect retained consumer ownership");
    let session_stop = enter
        .find("play_sessions().request_stop()")
        .expect("enter failure should stop the play session after successful cleanup");
    let compensated_shell_exit = enter
        .find("shell.state.exit_play_mode()")
        .expect("a successful compensating stop should restore the editor state");
    assert!(pending < session_stop);
    assert!(session_stop < compensated_shell_exit);
    assert!(
        enter.contains("MenuActionExecutionError::RuntimeConsumerStart"),
        "a retained consumer cleanup failure must preserve the typed startup error"
    );
    assert!(
        enter.contains("MenuActionExecutionError::RuntimeConsumerStartStopFailed"),
        "a failed compensating stop must preserve the play session instead of pretending it exited"
    );
    assert!(
        !enter.contains("let _ = controller.play_sessions().request_stop()"),
        "enter-play compensation must not discard a backend-stop failure"
    );
    assert!(
        enter.contains("MenuActionExecutionError::RuntimeConsumerStartRestoreStateFailed"),
        "a successful compensating stop must still report an editor-state restore failure"
    );
    assert!(
        !enter.contains("let _ = shell.state.exit_play_mode()"),
        "enter-play rollback must not discard an editor-state restore failure"
    );
    assert!(
        enter.contains("MenuActionExecutionError::PlayStartRestoreStateFailed"),
        "a failed backend start must report an editor-state restore failure"
    );

    let exit = source
        .split("MenuAction::ExitPlayMode =>")
        .nth(1)
        .expect("exit play mode action body");
    let consumer_retirement = exit
        .find("shutdown_runtime_event_consumers()")
        .expect("exit should perform one explicit consumer-retirement pass");
    let shell_exit = exit
        .find("shell.state.exit_play_mode()")
        .expect("exit should update shell state");
    let session_stop = exit
        .find("play_sessions()")
        .expect("exit should stop the play session");
    assert!(consumer_retirement < shell_exit);
    assert!(consumer_retirement < session_stop);
    assert!(session_stop < shell_exit);
    assert!(
        exit.contains("RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure"),
        "remote cleanup failure must be represented as local terminal retirement"
    );
    assert!(
        exit.contains("runtime event subscription cleanup is pending"),
        "the user-facing terminal status must retain the remote cleanup diagnostic"
    );
    assert!(
        exit.contains("RuntimeEventConsumerShutdownDisposition::RetirementDeferred"),
        "only a non-terminal local retirement may retain the editor in play mode"
    );
    assert!(
        !exit.contains("end_runtime_event_consumers()"),
        "exit must not make remote unsubscribe success a precondition for local retirement"
    );
    assert!(
        exit.contains("MenuActionExecutionError::PlayStopRestoreStateFailed"),
        "a successful stop must retain the failed editor-state restoration as a typed error"
    );
}

#[test]
fn runtime_event_pump_reports_editor_restore_failures_after_backend_stop() {
    let source = include_str!("../ui/host/editor_host_event_controller.rs");
    let stopped = source
        .split("if backend_transition.changed && !backend_transition.mode.has_active_runtime()")
        .nth(1)
        .and_then(|body| {
            body.split("return Ok(EditorRuntimeFrameDemand::OnDemand);")
                .next()
        })
        .expect("runtime-stop transition branch");

    assert!(
        stopped.contains("let editor_state_exit_error ="),
        "runtime-stop handling should retain the editor-state restore result"
    );
    assert!(
        stopped.contains("let pending_edit_decision_error ="),
        "runtime-stop handling should retain a decision-publication failure until state recovery runs"
    );
    assert!(
        !stopped.contains("let _ = shell.state.exit_play_mode()"),
        "runtime-stop handling must not discard an editor-state restore failure"
    );
    assert!(
        stopped.contains("editor state remains in play mode for retry"),
        "runtime-stop handling should expose a retryable editor-state restore failure"
    );
    assert!(
        stopped.contains("PlayTransitionCause::CleanupFailed"),
        "a stopped runtime with failed plugin restoration must leave Playing before UI recovery"
    );
    let reflection = stopped
        .find("self.refresh_reflection()")
        .expect("runtime-stop handling should refresh reflection state");
    let error = stopped
        .find("if let Some(source) = editor_state_exit_error")
        .expect("runtime-stop handling should return the retained restore error");
    assert!(
        reflection < error,
        "reflection should observe the backend stop even when editor-state restoration fails"
    );
    let decision_error = stopped
        .find("if let Some(decision_error) = pending_edit_decision_error")
        .expect("runtime-stop handling should return a retained decision-publication failure");
    assert!(
        reflection < decision_error,
        "a failed decision publication must not prevent state restoration and reflection refresh"
    );
    assert!(
        stopped.contains("PendingDecisionPublishAndStateRestore"),
        "dual decision and state-recovery failures should remain observable together"
    );
}
