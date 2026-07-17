use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use crate::core::gateway::{
    EditorRuntimeFrame, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
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
    deliveries: Mutex<BTreeMap<u64, Vec<ZrRuntimePluginEventDeliveryV1>>>,
    unsubscribed: Mutex<Vec<u64>>,
    unsubscribe_removed: Mutex<bool>,
    subscribe_calls: AtomicU64,
    fail_subscribe_call: Mutex<Option<u64>>,
}

impl FakeRuntimeGateway {
    fn new(session: u64) -> Self {
        Self {
            session: ZrRuntimeSessionHandle::new(session),
            deliveries: Mutex::new(BTreeMap::new()),
            unsubscribed: Mutex::new(Vec::new()),
            unsubscribe_removed: Mutex::new(true),
            subscribe_calls: AtomicU64::new(0),
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
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        Ok(self
            .deliveries
            .lock()
            .unwrap()
            .remove(&subscription.raw())
            .unwrap_or_default())
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
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationProgressV1, GatewayError> {
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
fn consumer_host_reports_runtime_refusing_to_remove_subscription() {
    let (host, client, state) = test_host();
    host.begin_play_session(400, &[CAPABILITY.to_string()])
        .unwrap();
    *client.unsubscribe_removed.lock().unwrap() = false;

    assert!(matches!(
        host.reconcile_enabled_capabilities(&[]),
        Err(EditorRuntimeEventConsumerError::Gateway { .. })
    ));
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(state.lock().unwrap().session, Some(400));

    *client.unsubscribe_removed.lock().unwrap() = true;
    host.reconcile_enabled_capabilities(&[]).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert!(state.lock().unwrap().session.is_none());
    host.end_play_session(400).unwrap();
}

#[test]
fn consumer_host_retains_failed_pie_exit_subscriptions_for_retry() {
    let (host, client, state) = test_host();
    host.begin_play_session(500, &[CAPABILITY.to_string()])
        .unwrap();
    *client.unsubscribe_removed.lock().unwrap() = false;

    assert!(matches!(
        host.end_play_session(500),
        Err(EditorRuntimeEventConsumerError::Gateway { .. })
    ));
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.active_play_session_id(), Some(500));
    assert_eq!(state.lock().unwrap().session, Some(500));

    *client.unsubscribe_removed.lock().unwrap() = true;
    host.end_play_session(500).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert!(state.lock().unwrap().session.is_none());
}

#[test]
fn consumer_host_reports_begin_rollback_failure_and_keeps_runtime_cleanup_retryable() {
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
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.active_play_session_id(), Some(600));

    *client.unsubscribe_removed.lock().unwrap() = true;
    host.end_play_session(600).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
}

#[test]
fn menu_action_keeps_runtime_alive_until_event_consumer_cleanup_succeeds() {
    let source = include_str!("../ui/host/editor_event_execution/menu_action.rs");
    let enter = source
        .split("MenuAction::EnterPlayMode =>")
        .nth(1)
        .and_then(|body| body.split("MenuAction::ExitPlayMode =>").next())
        .expect("enter play mode action body");
    let pending = enter
        .find("runtime_event_consumer_session_active()")
        .expect("enter failure should inspect retained consumer ownership");
    let backend_exit = enter
        .find(".backend().exit_play_mode()")
        .expect("enter failure should exit backend after successful cleanup");
    assert!(pending < backend_exit);
    assert!(enter.contains("runtime remains active so Exit Play can retry cleanup"));

    let exit = source
        .split("MenuAction::ExitPlayMode =>")
        .nth(1)
        .expect("exit play mode action body");
    let consumer_end = exit
        .find("end_runtime_event_consumers()")
        .expect("exit should clean up consumers");
    let shell_exit = exit
        .find("shell.state.exit_play_mode()")
        .expect("exit should update shell state");
    let backend_exit = exit
        .find(".backend().exit_play_mode()")
        .expect("exit should stop runtime backend");
    assert!(consumer_end < shell_exit);
    assert!(consumer_end < backend_exit);
    assert!(exit.contains("runtime remains active for retry"));
}
