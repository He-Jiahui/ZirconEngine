use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde::Deserialize;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use crate::core::gateway::{
    EditorRuntimeFrame, EditorRuntimeGateway, EditorRuntimePluginEventPage, GatewayError,
};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
    EditorRuntimeEventPumpBudget,
};

pub(super) const CAPABILITY: &str = "editor.tests.bounded-consumer";
pub(super) const SCHEMA: &str = "tests.events.bounded.v1";

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Payload {
    pub(super) value: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("test consumer rejected a payload")]
pub(super) struct ConsumerError;

#[derive(Default)]
pub(super) struct RecordingState {
    pub(super) session: Option<u64>,
    pub(super) sequences: Vec<u64>,
    pub(super) callback_delay: Duration,
}

impl EditorRuntimeEventConsumerState for RecordingState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, play_session_id: u64) {
        self.session = Some(play_session_id);
        self.sequences.clear();
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        assert_eq!(self.session, Some(play_session_id));
        assert_eq!(payload.value, sequence);
        if !self.callback_delay.is_zero() {
            std::thread::sleep(self.callback_delay);
        }
        self.sequences.push(sequence);
        Ok(())
    }

    fn end_session(&mut self, play_session_id: u64) {
        if self.session == Some(play_session_id) {
            self.session = None;
        }
    }
}

pub(super) struct ReentrantObservationState {
    pub(super) host: Weak<EditorRuntimeEventConsumerHost>,
    pub(super) observed_active: usize,
}

pub(super) struct ReentrantReconcileState {
    pub(super) host: Weak<EditorRuntimeEventConsumerHost>,
    pub(super) rejected_operation: Option<&'static str>,
}

impl EditorRuntimeEventConsumerState for ReentrantReconcileState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, _play_session_id: u64) {}

    fn consume(
        &mut self,
        _play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        let error = self
            .host
            .upgrade()
            .expect("host remains alive during callback")
            .reconcile_enabled_capabilities(&[])
            .expect_err("reentrant lifecycle mutation must be rejected without locking state");
        let EditorRuntimeEventConsumerError::LifecycleMutationBusy { operation } = error else {
            panic!("unexpected reentrant lifecycle error: {error}");
        };
        self.rejected_operation = Some(operation);
        Ok(())
    }

    fn end_session(&mut self, _play_session_id: u64) {}
}

pub(super) struct BlockingState {
    pub(super) entered: Arc<std::sync::Barrier>,
    pub(super) release: Arc<std::sync::Barrier>,
}

impl EditorRuntimeEventConsumerState for BlockingState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, _play_session_id: u64) {}

    fn consume(
        &mut self,
        _play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        self.entered.wait();
        self.release.wait();
        Ok(())
    }

    fn end_session(&mut self, _play_session_id: u64) {}
}

impl EditorRuntimeEventConsumerState for ReentrantObservationState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, _play_session_id: u64) {}

    fn consume(
        &mut self,
        _play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        self.observed_active = self
            .host
            .upgrade()
            .expect("host remains alive during callback")
            .active_consumer_count();
        Ok(())
    }

    fn end_session(&mut self, _play_session_id: u64) {}
}

pub(super) struct FakeGateway {
    session: ZrRuntimeSessionHandle,
    next_subscription: Mutex<u64>,
    deliveries: Mutex<BTreeMap<u64, Vec<ZrRuntimePluginEventDeliveryV1>>>,
    encoded_bytes: Mutex<BTreeMap<u64, usize>>,
    runtime_backlogs: Mutex<BTreeMap<u64, (usize, u64)>>,
    drain_calls: Mutex<BTreeMap<u64, usize>>,
    failing_drains: Mutex<BTreeSet<u64>>,
    failing_unsubscribes: Mutex<BTreeSet<u64>>,
    unsubscribed: Mutex<Vec<u64>>,
}

impl FakeGateway {
    pub(super) fn new(session: u64) -> Self {
        Self {
            session: ZrRuntimeSessionHandle::new(session),
            next_subscription: Mutex::new(10),
            deliveries: Mutex::new(BTreeMap::new()),
            encoded_bytes: Mutex::new(BTreeMap::new()),
            runtime_backlogs: Mutex::new(BTreeMap::new()),
            drain_calls: Mutex::new(BTreeMap::new()),
            failing_drains: Mutex::new(BTreeSet::new()),
            failing_unsubscribes: Mutex::new(BTreeSet::new()),
            unsubscribed: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn push(&self, subscription: u64, event_id: &str, sequence: u64) {
        self.deliveries
            .lock()
            .unwrap()
            .entry(subscription)
            .or_default()
            .push(ZrRuntimePluginEventDeliveryV1::new(
                self.session.raw(),
                ZrRuntimePluginEventSubscriptionHandle::new(subscription),
                event_id,
                SCHEMA,
                sequence,
                serde_json::json!({"value": sequence}),
            ));
    }

    pub(super) fn set_runtime_backlog(
        &self,
        subscription: u64,
        remaining_deliveries: usize,
        oldest_pending_age_millis: u64,
    ) {
        self.runtime_backlogs.lock().unwrap().insert(
            subscription,
            (remaining_deliveries, oldest_pending_age_millis),
        );
    }

    pub(super) fn set_encoded_bytes(&self, subscription: u64, encoded_bytes: usize) {
        self.encoded_bytes
            .lock()
            .unwrap()
            .insert(subscription, encoded_bytes);
    }

    pub(super) fn fail_drain(&self, subscription: u64) {
        self.failing_drains.lock().unwrap().insert(subscription);
    }

    pub(super) fn fail_unsubscribe(&self, subscription: u64) {
        self.failing_unsubscribes
            .lock()
            .unwrap()
            .insert(subscription);
    }

    pub(super) fn allow_unsubscribe(&self, subscription: u64) {
        self.failing_unsubscribes
            .lock()
            .unwrap()
            .remove(&subscription);
    }

    pub(super) fn drain_call_count(&self, subscription: u64) -> usize {
        self.drain_calls
            .lock()
            .unwrap()
            .get(&subscription)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn unsubscribed(&self) -> Vec<u64> {
        self.unsubscribed.lock().unwrap().clone()
    }
}

impl EditorRuntimeGateway for FakeGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::new(1, self.session, 1, None)
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
        let mut next = self.next_subscription.lock().unwrap();
        *next += 1;
        Ok(Some(ZrRuntimePluginEventSubscriptionHandle::new(*next)))
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.unsubscribed.lock().unwrap().push(subscription.raw());
        if self
            .failing_unsubscribes
            .lock()
            .unwrap()
            .contains(&subscription.raw())
        {
            return Err(GatewayError::Protocol {
                message: "injected unsubscribe failure".to_string(),
            });
        }
        Ok(true)
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        *self
            .drain_calls
            .lock()
            .unwrap()
            .entry(subscription.raw())
            .or_default() += 1;
        if self
            .failing_drains
            .lock()
            .unwrap()
            .contains(&subscription.raw())
        {
            return Err(GatewayError::Protocol {
                message: "injected drain failure".to_string(),
            });
        }
        let deliveries = self
            .deliveries
            .lock()
            .unwrap()
            .remove(&subscription.raw())
            .unwrap_or_default();
        let (remaining_deliveries, oldest_pending_age_millis) = self
            .runtime_backlogs
            .lock()
            .unwrap()
            .get(&subscription.raw())
            .copied()
            .unwrap_or_default();
        let encoded_bytes = self
            .encoded_bytes
            .lock()
            .unwrap()
            .get(&subscription.raw())
            .copied()
            .unwrap_or(subscription.raw() as usize * 10);
        Ok(EditorRuntimePluginEventPage::new(
            deliveries,
            encoded_bytes,
            Duration::ZERO,
            Duration::ZERO,
        )
        .with_runtime_backlog(remaining_deliveries, oldest_pending_age_millis))
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

pub(super) fn budget(
    max_events: usize,
    max_events_per_consumer: usize,
) -> EditorRuntimeEventPumpBudget {
    EditorRuntimeEventPumpBudget::new(
        max_events,
        max_events_per_consumer,
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
}

pub(super) fn register_state<S>(
    host: &EditorRuntimeEventConsumerHost,
    consumer_id: &str,
    event_id: &str,
    state: Arc<Mutex<S>>,
) where
    S: EditorRuntimeEventConsumerState + Sync,
{
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry
        .register(EditorRuntimeEventConsumerRegistration::typed(
            EditorRuntimeEventConsumerManifest::new(consumer_id, event_id, SCHEMA)
                .with_required_capability(CAPABILITY),
            state,
        ))
        .unwrap();
    host.register(registry).unwrap();
}

pub(super) fn percentile_index(sample_count: usize) -> usize {
    sample_count.saturating_mul(95).div_ceil(100) - 1
}
