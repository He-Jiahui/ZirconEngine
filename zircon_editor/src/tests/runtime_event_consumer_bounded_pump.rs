use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use serde::Deserialize;
use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8,
    ZrRuntimeEventV1, ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportSizeV1, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
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
static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);
static ABI_EVENT_ALLOCATIONS: OnceLock<Mutex<HashMap<u64, Box<[u8]>>>> = OnceLock::new();

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

unsafe extern "C" fn release_abi_event_page(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ABI_EVENT_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&allocation.raw())
        .expect("ABI event allocation must be released exactly once");
    ZrStatus::ok()
}

fn write_abi_event_page(batch: &ZrRuntimePluginEventDeliveryBatchV1, output: *mut ZrOwnedResultV2) {
    let bytes = serde_json::to_vec(batch)
        .expect("serialize bounded ABI event page")
        .into_boxed_slice();
    let data = bytes.as_ptr();
    let len = bytes.len() as u64;
    let allocation = ZrRuntimeAllocationId::new(NEXT_ALLOCATION_ID.fetch_add(1, Ordering::Relaxed));
    ABI_EVENT_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(allocation.raw(), bytes);
    unsafe {
        output.write(ZrOwnedResultV2 {
            data,
            len,
            allocation,
        })
    };
}

unsafe extern "C" fn abi_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedResultV2,
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

unsafe extern "C" fn request_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeViewportPickRequestV1,
    _out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn poll_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
    _out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn cancel_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

fn abi_gateway() -> SessionGateway {
    let mut api = ZrRuntimeApiV8::empty();
    api.release_allocation = Some(release_abi_event_page);
    api.subscribe_plugin_event = Some(abi_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(abi_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(abi_drain_plugin_events);
    api.request_viewport_pick = Some(request_test_viewport_pick);
    api.poll_viewport_pick = Some(poll_test_viewport_pick);
    api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
    unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(7),
            GatewaySessionIdentity::new(7, ZrRuntimeSessionHandle::new(7), 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
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
    encoded_bytes: Mutex<BTreeMap<u64, usize>>,
    runtime_backlogs: Mutex<BTreeMap<u64, (usize, u64)>>,
    drain_calls: Mutex<BTreeMap<u64, usize>>,
    failing_drains: Mutex<BTreeSet<u64>>,
    failing_unsubscribes: Mutex<BTreeSet<u64>>,
    unsubscribed: Mutex<Vec<u64>>,
}

impl FakeGateway {
    fn new(session: u64) -> Self {
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

    fn set_runtime_backlog(
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

    fn set_encoded_bytes(&self, subscription: u64, encoded_bytes: usize) {
        self.encoded_bytes
            .lock()
            .unwrap()
            .insert(subscription, encoded_bytes);
    }

    fn fail_drain(&self, subscription: u64) {
        self.failing_drains.lock().unwrap().insert(subscription);
    }

    fn fail_unsubscribe(&self, subscription: u64) {
        self.failing_unsubscribes
            .lock()
            .unwrap()
            .insert(subscription);
    }

    fn allow_unsubscribe(&self, subscription: u64) {
        self.failing_unsubscribes
            .lock()
            .unwrap()
            .remove(&subscription);
    }

    fn drain_call_count(&self, subscription: u64) -> usize {
        self.drain_calls
            .lock()
            .unwrap()
            .get(&subscription)
            .copied()
            .unwrap_or_default()
    }

    fn unsubscribed(&self) -> Vec<u64> {
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
            return Err("injected unsubscribe failure".to_string());
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
    const MAX_EVENTS_PER_TICK: usize = 32;

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
    let mut max_editor_pending_encoded_bytes_upper_bound = 0_usize;
    let mut max_editor_pending_oldest_age_millis = 0_u128;
    let mut last_observed_runtime_remaining_peak = 0_usize;
    let mut max_last_observed_runtime_oldest_pending_age_millis = 0_u64;
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
        max_editor_pending_encoded_bytes_upper_bound = max_editor_pending_encoded_bytes_upper_bound
            .max(report.pending_encoded_bytes_upper_bound());
        max_editor_pending_oldest_age_millis =
            max_editor_pending_oldest_age_millis.max(report.pending_oldest_age().as_millis());
        let runtime_backlog = report.runtime_backlog_observation();
        last_observed_runtime_remaining_peak = last_observed_runtime_remaining_peak
            .max(runtime_backlog.known_remaining_deliveries_lower_bound());
        if let Some(oldest_pending_age_millis) = runtime_backlog.max_oldest_pending_age_millis() {
            max_last_observed_runtime_oldest_pending_age_millis =
                max_last_observed_runtime_oldest_pending_age_millis.max(oldest_pending_age_millis);
        }
        assert!(report.applied() <= MAX_EVENTS_PER_TICK);
        assert!(report.drained() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1);
        assert!(
            report.drained_encoded_bytes() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert!(
            report.pending_encoded_bytes_upper_bound()
                <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert_eq!(report.runtime_drain_p95(), report.runtime_drain_elapsed());
        assert_eq!(report.decode_p95(), report.decode_elapsed());
        assert_eq!(report.dropped(), 0);
        if report.drained() > 0 {
            assert_eq!(runtime_backlog.sampled_consumer_count(), 1);
            assert_eq!(runtime_backlog.unknown_consumer_count(), 0);
            let remaining = runtime_backlog.known_remaining_deliveries_lower_bound();
            assert_eq!(
                remaining.saturating_add(report.queue_depth()),
                delivery_count as usize - applied
            );
            assert_eq!(
                runtime_backlog.max_oldest_pending_age_millis(),
                Some(if remaining == 0 { 0 } else { 17 })
            );
            assert!(runtime_backlog.max_observation_age().is_some());
        }
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
        "max_editor_pending_encoded_bytes_upper_bound": max_editor_pending_encoded_bytes_upper_bound,
        "max_editor_pending_oldest_age_millis": max_editor_pending_oldest_age_millis,
        "last_observed_runtime_remaining_peak": last_observed_runtime_remaining_peak,
        "max_last_observed_runtime_oldest_pending_age_millis": max_last_observed_runtime_oldest_pending_age_millis,
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

#[path = "runtime_event_consumer_bounded_pump/round_robin.rs"]
mod round_robin;

#[path = "runtime_event_consumer_bounded_pump/faults.rs"]
mod faults;

#[path = "runtime_event_consumer_bounded_pump/pumping.rs"]
mod pumping;
