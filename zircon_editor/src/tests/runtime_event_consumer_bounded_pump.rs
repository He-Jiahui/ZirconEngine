use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use serde::Deserialize;
use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1, ZrByteSlice, ZrOwnedByteBuffer,
    ZrRuntimeApiV3, ZrRuntimeEventV1, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
};

use crate::core::gateway::{
    EditorRuntimeFrame, EditorRuntimeGateway, EditorRuntimeGatewayHandle,
    EditorRuntimePluginEventPage, GatewayError, RuntimeCapabilities, SessionGateway,
};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
    EditorRuntimeEventPumpBudget,
};

const CAPABILITY: &str = "editor.tests.bounded-consumer";
const SCHEMA: &str = "tests.events.bounded.v1";

#[derive(Default)]
struct AbiEventBacklog {
    remaining: u64,
    next_sequence: u64,
    oldest_pending_age_millis: u64,
}

static ABI_EVENT_BACKLOG: Mutex<AbiEventBacklog> = Mutex::new(AbiEventBacklog {
    remaining: 0,
    next_sequence: 1,
    oldest_pending_age_millis: 0,
});
static ABI_EVENT_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

unsafe extern "C" fn abi_subscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    output: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    output.write(ZrRuntimePluginEventSubscriptionHandle::new(11));
    ZrStatus::ok()
}

unsafe extern "C" fn abi_unsubscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn free_abi_event_page(output: ZrOwnedByteBuffer) -> ZrStatus {
    drop(unsafe { Vec::from_raw_parts(output.data, output.len, output.capacity) });
    ZrStatus::ok()
}

fn write_abi_event_page(
    batch: &ZrRuntimePluginEventDeliveryBatchV1,
    output: *mut ZrOwnedByteBuffer,
) {
    let mut bytes = serde_json::to_vec(batch).expect("serialize bounded ABI event page");
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 0,
        free: Some(free_abi_event_page),
    };
    std::mem::forget(bytes);
    unsafe { output.write(buffer) };
}

unsafe extern "C" fn abi_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    let mut backlog = ABI_EVENT_BACKLOG.lock().expect("lock ABI event backlog");
    let count = backlog
        .remaining
        .min(ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 as u64);
    let first_sequence = backlog.next_sequence;
    backlog.remaining -= count;
    backlog.next_sequence = backlog.next_sequence.saturating_add(count);
    let remaining_deliveries = u32::try_from(backlog.remaining)
        .expect("bounded ABI event fixture remaining count fits the protocol field");
    let oldest_pending_age_millis = if remaining_deliveries == 0 {
        0
    } else {
        backlog.oldest_pending_age_millis
    };
    drop(backlog);
    let deliveries = (first_sequence..first_sequence.saturating_add(count))
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                7,
                subscription,
                "tests.events.storm",
                SCHEMA,
                sequence,
                serde_json::json!({"value": sequence}),
            )
        })
        .collect();
    write_abi_event_page(
        &ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries)
            .with_runtime_backlog(remaining_deliveries, oldest_pending_age_millis),
        output,
    );
    ZrStatus::ok()
}

fn abi_gateway() -> SessionGateway {
    let mut api = ZrRuntimeApiV3::empty();
    api.subscribe_plugin_event = Some(abi_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(abi_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(abi_drain_plugin_events);
    unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(7),
            RuntimeCapabilities::editor_default(),
        )
        .expect("construct bounded ABI session gateway")
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Payload {
    value: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("test consumer rejected a payload")]
struct ConsumerError;

#[derive(Default)]
struct RecordingState {
    session: Option<u64>,
    sequences: Vec<u64>,
    callback_delay: Duration,
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

struct ReentrantObservationState {
    host: Weak<EditorRuntimeEventConsumerHost>,
    observed_active: usize,
}

struct ReentrantReconcileState {
    host: Weak<EditorRuntimeEventConsumerHost>,
    rejected_operation: Option<&'static str>,
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

struct BlockingState {
    entered: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
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

struct FakeGateway {
    session: ZrRuntimeSessionHandle,
    next_subscription: Mutex<u64>,
    deliveries: Mutex<BTreeMap<u64, Vec<ZrRuntimePluginEventDeliveryV1>>>,
    failing_drains: Mutex<BTreeSet<u64>>,
}

impl FakeGateway {
    fn new(session: u64) -> Self {
        Self {
            session: ZrRuntimeSessionHandle::new(session),
            next_subscription: Mutex::new(10),
            deliveries: Mutex::new(BTreeMap::new()),
            failing_drains: Mutex::new(BTreeSet::new()),
        }
    }

    fn push(&self, subscription: u64, event_id: &str, sequence: u64) {
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

    fn fail_drain(&self, subscription: u64) {
        self.failing_drains.lock().unwrap().insert(subscription);
    }
}

impl EditorRuntimeGateway for FakeGateway {
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
        let mut next = self.next_subscription.lock().unwrap();
        *next += 1;
        Ok(Some(ZrRuntimePluginEventSubscriptionHandle::new(*next)))
    }

    fn unsubscribe_plugin_event(
        &self,
        _subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        Ok(true)
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
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

fn budget(max_events: usize, max_events_per_consumer: usize) -> EditorRuntimeEventPumpBudget {
    EditorRuntimeEventPumpBudget::new(
        max_events,
        max_events_per_consumer,
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
}

fn register_state<S>(
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

#[test]
fn bounded_pump_defers_backlog_without_losing_order() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", state.clone());
    host.begin_play_session(100, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in 1..=10 {
        gateway.push(11, "tests.events.a", sequence);
    }

    let first = host.pump_with_budget(budget(3, 3)).unwrap();
    assert_eq!(first.applied(), 3);
    assert_eq!(first.drained(), 10);
    assert_eq!(first.drained_encoded_bytes(), 0);
    assert_eq!(first.deferred(), 7);
    assert_eq!(first.queue_depth(), 7);
    assert_eq!(state.lock().unwrap().sequences, [1, 2, 3]);

    while host.last_pump_report().queue_depth() != 0 {
        host.pump_with_budget(budget(3, 3)).unwrap();
    }
    assert_eq!(
        state.lock().unwrap().sequences,
        (1..=10).collect::<Vec<_>>()
    );
}

#[test]
fn round_robin_budget_gives_each_consumer_a_turn() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let first = Arc::new(Mutex::new(RecordingState::default()));
    let second = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", first.clone());
    register_state(&host, "tests.consumer.b", "tests.events.b", second.clone());
    host.begin_play_session(200, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in 1..=4 {
        gateway.push(11, "tests.events.a", sequence);
        gateway.push(12, "tests.events.b", sequence);
    }

    host.pump_with_budget(budget(1, 1)).unwrap();
    host.pump_with_budget(budget(1, 1)).unwrap();

    assert_eq!(first.lock().unwrap().sequences, [1]);
    assert_eq!(second.lock().unwrap().sequences, [1]);
}

#[test]
fn round_robin_start_rotates_under_non_divisible_budgets() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let first = Arc::new(Mutex::new(RecordingState::default()));
    let second = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", first.clone());
    register_state(&host, "tests.consumer.b", "tests.events.b", second.clone());
    host.begin_play_session(225, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in 1..=6 {
        gateway.push(11, "tests.events.a", sequence);
        gateway.push(12, "tests.events.b", sequence);
    }

    host.pump_with_budget(budget(3, 2)).unwrap();
    assert_eq!(first.lock().unwrap().sequences, [1, 2]);
    assert_eq!(second.lock().unwrap().sequences, [1]);

    host.pump_with_budget(budget(3, 2)).unwrap();
    assert_eq!(first.lock().unwrap().sequences, [1, 2, 3]);
    assert_eq!(second.lock().unwrap().sequences, [1, 2, 3]);
}

#[test]
fn gateway_failure_does_not_starve_later_consumers() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let first = Arc::new(Mutex::new(RecordingState::default()));
    let second = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", first.clone());
    register_state(&host, "tests.consumer.b", "tests.events.b", second.clone());
    host.begin_play_session(250, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.fail_drain(11);
    gateway.push(12, "tests.events.b", 1);

    let error = host
        .pump_with_budget(budget(1, 1))
        .expect_err("the first gateway error remains observable");

    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::Gateway { .. }
    ));
    assert!(first.lock().unwrap().sequences.is_empty());
    assert_eq!(second.lock().unwrap().sequences, [1]);
}

#[test]
fn consumer_callback_can_reenter_host_observation_without_deadlock() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = Arc::new(EditorRuntimeEventConsumerHost::new(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
    ));
    let state = Arc::new(Mutex::new(ReentrantObservationState {
        host: Arc::downgrade(&host),
        observed_active: 0,
    }));
    register_state(
        &host,
        "tests.consumer.reentrant",
        "tests.events.reentrant",
        state.clone(),
    );
    host.begin_play_session(300, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.reentrant", 1);

    let (sender, receiver) = std::sync::mpsc::channel();
    let pump_host = host.clone();
    std::thread::spawn(move || sender.send(pump_host.pump()).unwrap());
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("reentrant observation must not deadlock")
            .unwrap(),
        1
    );
    assert_eq!(state.lock().unwrap().observed_active, 1);
}

#[test]
fn consumer_callback_reconcile_is_typed_busy_without_deadlock() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = Arc::new(EditorRuntimeEventConsumerHost::new(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
    ));
    let state = Arc::new(Mutex::new(ReentrantReconcileState {
        host: Arc::downgrade(&host),
        rejected_operation: None,
    }));
    register_state(
        &host,
        "tests.consumer.reentrant-reconcile",
        "tests.events.reentrant-reconcile",
        state.clone(),
    );
    host.begin_play_session(350, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.reentrant-reconcile", 1);

    let (sender, receiver) = std::sync::mpsc::channel();
    let pump_host = host.clone();
    std::thread::spawn(move || sender.send(pump_host.pump()).unwrap());
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("reentrant reconcile rejection must not deadlock")
            .unwrap(),
        1
    );
    assert_eq!(
        state.lock().unwrap().rejected_operation,
        Some("reconcile enabled capabilities")
    );
    assert_eq!(host.active_consumer_count(), 1);
}

#[test]
fn concurrent_end_session_is_typed_busy_until_pump_releases_owner() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = Arc::new(EditorRuntimeEventConsumerHost::new(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
    ));
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let state = Arc::new(Mutex::new(BlockingState {
        entered: entered.clone(),
        release: release.clone(),
    }));
    register_state(
        &host,
        "tests.consumer.concurrent-lifecycle",
        "tests.events.concurrent-lifecycle",
        state,
    );
    host.begin_play_session(375, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.concurrent-lifecycle", 1);

    let pump_host = host.clone();
    let pump = std::thread::spawn(move || pump_host.pump());
    entered.wait();
    let error = host
        .end_play_session(375)
        .expect_err("lifecycle mutation must not race the active pump owner");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::LifecycleMutationBusy {
            operation: "end play session"
        }
    ));
    assert_eq!(host.active_consumer_count(), 1);
    release.wait();
    assert_eq!(pump.join().unwrap().unwrap(), 1);
    host.end_play_session(375).unwrap();
}

#[test]
fn slow_callback_is_visible_in_pump_report() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let state = Arc::new(Mutex::new(RecordingState {
        callback_delay: Duration::from_millis(2),
        ..RecordingState::default()
    }));
    register_state(&host, "tests.consumer.slow", "tests.events.slow", state);
    host.begin_play_session(400, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.slow", 1);

    let report = host
        .pump_with_budget(EditorRuntimeEventPumpBudget::new(
            4,
            4,
            Duration::from_secs(1),
            Duration::from_millis(1),
        ))
        .unwrap();

    assert_eq!(report.applied(), 1);
    assert_eq!(report.slow_callbacks(), 1);
}

#[test]
#[ignore = "managed performance evidence; run alone with --test-threads=1"]
fn managed_thousand_and_ten_thousand_delivery_budget_report() {
    let _fixture_guard = ABI_EVENT_FIXTURE_LOCK
        .lock()
        .expect("lock ABI event fixture");
    let reports = [1_000_u64, 10_000]
        .into_iter()
        .map(run_abi_delivery_storm)
        .collect::<Vec<_>>();
    println!(
        "PLUGINS01_RUNTIME_EVENT_ABI_PUMP_BENCHMARK={}",
        serde_json::Value::Array(reports)
    );
}

fn run_abi_delivery_storm(delivery_count: u64) -> serde_json::Value {
    const MAX_EVENTS_PER_TICK: usize = 64;

    *ABI_EVENT_BACKLOG.lock().expect("lock ABI event backlog") = AbiEventBacklog {
        remaining: delivery_count,
        next_sequence: 1,
        oldest_pending_age_millis: 17,
    };

    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(Arc::new(
        abi_gateway(),
    )));
    let state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.storm",
        "tests.events.storm",
        state.clone(),
    );
    host.begin_play_session(500, &[CAPABILITY.to_string()])
        .unwrap();
    let mut tick_durations = Vec::new();
    let mut runtime_drain_durations = Vec::new();
    let mut decode_durations = Vec::new();
    let mut applied = 0_usize;
    let mut max_applied_per_tick = 0_usize;
    let mut max_drained_per_tick = 0_usize;
    let mut max_page_bytes = 0_usize;
    let mut pending_peak = 0_usize;
    let mut max_pending_sequence_span = 0_u64;
    let mut runtime_remaining_peak = 0_usize;
    let mut max_runtime_oldest_pending_age_millis = 0_u64;
    while applied < delivery_count as usize {
        let started = Instant::now();
        let report = host
            .pump_with_budget(EditorRuntimeEventPumpBudget::new(
                MAX_EVENTS_PER_TICK,
                MAX_EVENTS_PER_TICK,
                Duration::from_secs(1),
                Duration::from_millis(1),
            ))
            .unwrap();
        tick_durations.push(started.elapsed());
        runtime_drain_durations.push(report.runtime_drain_elapsed());
        decode_durations.push(report.decode_elapsed());
        applied = applied.saturating_add(report.applied());
        max_applied_per_tick = max_applied_per_tick.max(report.applied());
        max_drained_per_tick = max_drained_per_tick.max(report.drained());
        max_page_bytes = max_page_bytes.max(report.drained_encoded_bytes());
        pending_peak = pending_peak.max(report.queue_depth());
        max_pending_sequence_span = max_pending_sequence_span.max(report.pending_sequence_span());
        runtime_remaining_peak = runtime_remaining_peak.max(report.runtime_remaining_deliveries());
        max_runtime_oldest_pending_age_millis =
            max_runtime_oldest_pending_age_millis.max(report.runtime_oldest_pending_age_millis());
        assert!(report.applied() <= MAX_EVENTS_PER_TICK);
        assert!(report.drained() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1);
        assert!(
            report.drained_encoded_bytes() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert_eq!(report.runtime_drain_p95(), report.runtime_drain_elapsed());
        assert_eq!(report.decode_p95(), report.decode_elapsed());
        assert_eq!(report.dropped(), 0);
        assert_eq!(
            report.runtime_remaining_deliveries(),
            delivery_count as usize - applied
        );
        assert_eq!(
            report.runtime_oldest_pending_age_millis(),
            if report.runtime_remaining_deliveries() == 0 {
                0
            } else {
                17
            }
        );
    }

    assert_eq!(applied, delivery_count as usize);
    assert_eq!(
        state.lock().unwrap().sequences.len(),
        delivery_count as usize
    );
    assert_eq!(
        ABI_EVENT_BACKLOG
            .lock()
            .expect("lock ABI event backlog")
            .remaining,
        0
    );
    tick_durations.sort_unstable();
    runtime_drain_durations.sort_unstable();
    decode_durations.sort_unstable();
    let p95_index = percentile_index(tick_durations.len());
    let tick_p95_ns = u64::try_from(tick_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);
    let runtime_drain_p95_ns =
        u64::try_from(runtime_drain_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);
    let decode_p95_ns = u64::try_from(decode_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);

    serde_json::json!({
        "deliveries": delivery_count,
        "ticks": tick_durations.len(),
        "max_events_per_tick": MAX_EVENTS_PER_TICK,
        "max_applied_per_tick": max_applied_per_tick,
        "max_drained_per_tick": max_drained_per_tick,
        "max_page_bytes": max_page_bytes,
        "pending_peak": pending_peak,
        "max_pending_sequence_span": max_pending_sequence_span,
        "runtime_remaining_peak": runtime_remaining_peak,
        "max_runtime_oldest_pending_age_millis": max_runtime_oldest_pending_age_millis,
        "tick_p95_ns": tick_p95_ns,
        "runtime_drain_p95_ns": runtime_drain_p95_ns,
        "decode_p95_ns": decode_p95_ns,
        "applied": applied,
        "dropped": host.last_pump_report().dropped(),
        "remaining_queue_depth": host.last_pump_report().queue_depth(),
    })
}

fn percentile_index(sample_count: usize) -> usize {
    sample_count.saturating_mul(95).div_ceil(100) - 1
}

#[path = "runtime_event_consumer_bounded_pump/real_runtime_abi.rs"]
mod real_runtime_abi;
