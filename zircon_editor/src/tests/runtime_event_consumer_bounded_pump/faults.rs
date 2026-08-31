use std::sync::{Arc, Mutex};

use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerCallbackPhase, EditorRuntimeEventConsumerDeliveryDisposition,
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerFaultPolicy,
    EditorRuntimeEventConsumerFaultReceiptBudget, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerPendingDeliveryBudget, EditorRuntimeEventConsumerQuarantineReason,
    EditorRuntimeEventConsumerRetentionBudget, EditorRuntimeEventConsumerState,
    EditorRuntimeEventPumpBudget,
};

use super::{
    budget, register_state, ConsumerError, FakeGateway, Payload, RecordingState, CAPABILITY,
};

struct PanicOnceState {
    session: Option<u64>,
    sequences: Vec<u64>,
    panic_on_sequence: u64,
    panicked: bool,
}

impl EditorRuntimeEventConsumerState for PanicOnceState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, play_session_id: u64) {
        self.session = Some(play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        assert_eq!(self.session, Some(play_session_id));
        assert_eq!(payload.value, sequence);
        if sequence == self.panic_on_sequence && !self.panicked {
            self.panicked = true;
            panic!("injected consumer callback panic");
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

struct PanicOnBeginState;

impl EditorRuntimeEventConsumerState for PanicOnBeginState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, _play_session_id: u64) {
        panic!("injected begin callback panic");
    }

    fn consume(
        &mut self,
        _play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_session(&mut self, _play_session_id: u64) {}
}

struct PanicOnEndState;

struct RejectingState {
    session: Option<u64>,
}

impl EditorRuntimeEventConsumerState for RejectingState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, play_session_id: u64) {
        self.session = Some(play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        assert_eq!(self.session, Some(play_session_id));
        Err(ConsumerError)
    }

    fn end_session(&mut self, play_session_id: u64) {
        if self.session == Some(play_session_id) {
            self.session = None;
        }
    }
}

impl EditorRuntimeEventConsumerState for PanicOnEndState {
    type Payload = Payload;
    type Error = ConsumerError;

    fn begin_session(&mut self, _play_session_id: u64) {}

    fn consume(
        &mut self,
        _play_session_id: u64,
        _sequence: u64,
        _payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_session(&mut self, _play_session_id: u64) {
        panic!("injected end callback panic");
    }
}

#[test]
fn consume_panic_is_typed_and_does_not_starve_other_consumers() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let panic_state = Arc::new(Mutex::new(PanicOnceState {
        session: None,
        sequences: Vec::new(),
        panic_on_sequence: 3,
        panicked: false,
    }));
    let healthy_state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.panic",
        "tests.events.panic",
        panic_state.clone(),
    );
    register_state(
        &host,
        "tests.consumer.healthy",
        "tests.events.healthy",
        healthy_state.clone(),
    );
    host.begin_play_session(101, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in [1, 2, 3, 2, 4] {
        gateway.push(12, "tests.events.panic", sequence);
    }
    gateway.push(11, "tests.events.healthy", 1);

    let error = host
        .pump_with_budget(budget(6, 5))
        .expect_err("a consumer panic must become a typed host error");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::CallbackPanicked {
            phase: EditorRuntimeEventConsumerCallbackPhase::Consume,
            delivery_sequence: Some(3),
            ..
        }
    ));
    assert_eq!(gateway.drain_call_count(12), 1);
    assert_eq!(healthy_state.lock().unwrap().sequences, [1]);
    assert_eq!(panic_state.lock().unwrap().sequences, [1, 2]);
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.quarantined_consumer_count(), 1);
    assert_eq!(host.last_pump_report().dropped(), 3);
    assert_eq!(gateway.unsubscribed(), [12]);

    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].consumer_id(), "tests.consumer.panic");
    assert_eq!(receipts[0].play_session_id(), 101);
    assert_eq!(
        receipts[0].phase(),
        EditorRuntimeEventConsumerCallbackPhase::Consume
    );
    assert_eq!(receipts[0].delivery_sequence(), Some(3));
    assert_eq!(
        receipts[0].delivery_disposition(),
        Some(EditorRuntimeEventConsumerDeliveryDisposition::Poison)
    );
    assert_eq!(receipts[0].event_id(), Some("tests.events.panic"));
    assert_eq!(
        receipts[0].payload_schema(),
        Some("tests.events.bounded.v1")
    );
    assert!(receipts[0]
        .payload_json()
        .is_some_and(|payload| payload.contains("\"value\":3")));

    let quarantined = host.pump_with_budget(budget(5, 5)).unwrap();
    assert_eq!(quarantined.applied(), 0);
    assert_eq!(gateway.drain_call_count(12), 1);
    assert_eq!(panic_state.lock().unwrap().sequences, [1, 2]);

    host.end_play_session(101).unwrap();
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.quarantined_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert_eq!(gateway.unsubscribed(), [12, 11]);
}

#[test]
fn begin_panic_quarantines_only_the_faulting_consumer_and_keeps_the_session_running() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    register_state(
        &host,
        "tests.consumer.begin-panic",
        "tests.events.begin-panic",
        Arc::new(Mutex::new(PanicOnBeginState)),
    );
    let healthy = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.healthy",
        "tests.events.healthy",
        healthy.clone(),
    );

    host.begin_play_session(102, &[CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.quarantined_consumer_count(), 1);
    assert_eq!(host.active_play_session_id(), Some(102));
    assert_eq!(gateway.unsubscribed(), [11]);
    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0].phase(),
        EditorRuntimeEventConsumerCallbackPhase::BeginSession
    );
    assert_eq!(receipts[0].delivery_sequence(), None);
    assert_eq!(receipts[0].delivery_disposition(), None);

    gateway.push(12, "tests.events.healthy", 1);
    assert_eq!(host.pump().unwrap(), 1);
    assert_eq!(healthy.lock().unwrap().sequences, [1]);

    host.end_play_session(102).unwrap();
    assert_eq!(gateway.unsubscribed(), [11, 12]);
}

#[test]
fn begin_panic_records_failed_remote_cleanup_for_reconciliation() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    register_state(
        &host,
        "tests.consumer.begin-panic",
        "tests.events.begin-panic",
        Arc::new(Mutex::new(PanicOnBeginState)),
    );
    gateway.fail_unsubscribe(11);

    host.begin_play_session(106, &[CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(host.active_play_session_id(), Some(106));
    assert_eq!(host.quarantined_consumer_count(), 1);
    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].remote_cleanup_error().is_some());
    assert_eq!(gateway.unsubscribed(), [11]);

    gateway.allow_unsubscribe(11);
    host.reconcile_enabled_capabilities(&[CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(gateway.unsubscribed(), [11, 11]);
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.quarantined_consumer_count(), 1);
    host.end_play_session(106).unwrap();
    assert_eq!(host.active_play_session_id(), None);
    assert_eq!(host.quarantined_consumer_count(), 0);
}

#[test]
fn end_panic_retires_the_consumer_before_reporting_the_fault() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    register_state(
        &host,
        "tests.consumer.end-panic",
        "tests.events.end-panic",
        Arc::new(Mutex::new(PanicOnEndState)),
    );
    host.begin_play_session(103, &[CAPABILITY.to_string()])
        .unwrap();

    let error = host
        .end_play_session(103)
        .expect_err("an end panic must remain observable");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::CallbackPanicked {
            phase: EditorRuntimeEventConsumerCallbackPhase::EndSession,
            delivery_sequence: None,
            ..
        }
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert_eq!(gateway.unsubscribed(), [11]);
    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0].phase(),
        EditorRuntimeEventConsumerCallbackPhase::EndSession
    );
}

#[test]
fn remote_unsubscribe_failure_does_not_leave_a_consumer_active() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    register_state(
        &host,
        "tests.consumer.unsubscribe-failure",
        "tests.events.unsubscribe-failure",
        Arc::new(Mutex::new(RecordingState::default())),
    );
    host.begin_play_session(104, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.fail_unsubscribe(11);

    let error = host
        .end_play_session(104)
        .expect_err("remote cleanup failure remains observable after local retirement");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::Gateway { .. }
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.active_play_session_id(), None);
    assert_eq!(gateway.unsubscribed(), [11]);
}

#[test]
fn repeated_callback_failures_quarantine_only_the_failing_consumer() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::with_fault_policy(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
        EditorRuntimeEventConsumerFaultPolicy::new(2, 8, 8, 8),
    );
    let state = Arc::new(Mutex::new(RejectingState { session: None }));
    register_state(
        &host,
        "tests.consumer.repeated-failure",
        "tests.events.repeated-failure",
        state.clone(),
    );
    host.begin_play_session(107, &[CAPABILITY.to_string()])
        .unwrap();

    gateway.push(11, "tests.events.repeated-failure", 1);
    assert!(matches!(
        host.pump_with_budget(budget(4, 4)),
        Err(EditorRuntimeEventConsumerError::Payload { .. })
    ));
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.quarantined_consumer_count(), 0);

    gateway.push(11, "tests.events.repeated-failure", 2);
    assert!(matches!(
        host.pump_with_budget(budget(4, 4)),
        Err(EditorRuntimeEventConsumerError::Payload { .. })
    ));
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(host.quarantined_consumer_count(), 1);
    assert_eq!(
        host.quarantined_consumer_reason("tests.consumer.repeated-failure"),
        Some(EditorRuntimeEventConsumerQuarantineReason::ConsecutiveFailures)
    );
    assert_eq!(state.lock().unwrap().session, None);
    assert_eq!(gateway.unsubscribed(), [11]);

    host.disable_quarantined_consumer("tests.consumer.repeated-failure")
        .unwrap();
    assert!(host.consumer_is_user_disabled("tests.consumer.repeated-failure"));
    host.retry_quarantined_consumer("tests.consumer.repeated-failure", &[CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(host.active_consumer_count(), 1);
    assert_eq!(host.quarantined_consumer_count(), 0);
    assert!(!host.consumer_is_user_disabled("tests.consumer.repeated-failure"));
    assert_eq!(gateway.unsubscribed(), [11]);
}

#[test]
fn repeated_slow_callbacks_retire_after_the_committed_delivery() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::with_fault_policy(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
        EditorRuntimeEventConsumerFaultPolicy::new(8, 8, 8, 1),
    );
    let state = Arc::new(Mutex::new(RecordingState {
        callback_delay: std::time::Duration::from_millis(2),
        ..RecordingState::default()
    }));
    register_state(
        &host,
        "tests.consumer.repeated-slow",
        "tests.events.repeated-slow",
        state.clone(),
    );
    host.begin_play_session(108, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.repeated-slow", 1);

    let report = host
        .pump_with_budget(EditorRuntimeEventPumpBudget::new(
            4,
            4,
            std::time::Duration::from_secs(1),
            std::time::Duration::from_millis(1),
        ))
        .unwrap();
    assert_eq!(report.applied(), 1);
    assert_eq!(report.slow_callbacks(), 1);
    assert_eq!(host.active_consumer_count(), 0);
    assert_eq!(
        host.quarantined_consumer_reason("tests.consumer.repeated-slow"),
        Some(EditorRuntimeEventConsumerQuarantineReason::ConsecutiveSlowCallbacks)
    );
    assert_eq!(state.lock().unwrap().session, None);
    assert_eq!(gateway.unsubscribed(), [11]);
}

#[test]
fn callback_fault_receipts_evict_old_payloads_within_the_configured_budget() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::with_fault_receipt_budget(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
        EditorRuntimeEventConsumerFaultReceiptBudget::new(1, 1),
    );
    for (consumer_id, event_id) in [
        ("tests.consumer.panic-a", "tests.events.panic-a"),
        ("tests.consumer.panic-b", "tests.events.panic-b"),
    ] {
        register_state(
            &host,
            consumer_id,
            event_id,
            Arc::new(Mutex::new(PanicOnceState {
                session: None,
                sequences: Vec::new(),
                panic_on_sequence: 1,
                panicked: false,
            })),
        );
    }
    host.begin_play_session(105, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.panic-a", 1);
    gateway.push(12, "tests.events.panic-b", 1);

    let error = host
        .pump_with_budget(budget(4, 4))
        .expect_err("both panics are still surfaced through the host error path");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::CallbackPanicked { .. }
    ));
    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].consumer_id(), "tests.consumer.panic-b");
    assert!(receipts[0].payload_json().is_none());
    assert!(receipts[0].payload_was_truncated());
    assert_eq!(host.quarantined_consumer_count(), 2);
}

#[test]
fn pending_tail_and_fault_payload_share_one_retained_byte_budget() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::with_budgets_and_retention(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
        EditorRuntimeEventConsumerFaultReceiptBudget::new(2, 256),
        EditorRuntimeEventConsumerPendingDeliveryBudget::new(240),
        EditorRuntimeEventConsumerRetentionBudget::new(190),
    );
    let pending_state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.a-pending",
        "tests.events.a-pending",
        pending_state.clone(),
    );
    register_state(
        &host,
        "tests.consumer.b-panic",
        "tests.events.b-panic",
        Arc::new(Mutex::new(PanicOnceState {
            session: None,
            sequences: Vec::new(),
            panic_on_sequence: 1,
            panicked: false,
        })),
    );
    host.begin_play_session(109, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.set_encoded_bytes(11, 120);
    gateway.set_encoded_bytes(12, 120);
    gateway.push(11, "tests.events.a-pending", 1);
    gateway.push(11, "tests.events.a-pending", 2);
    gateway.push(12, "tests.events.b-panic", 1);

    let first = host.pump_with_budget(budget(1, 1)).unwrap();
    assert_eq!(first.applied(), 1);
    assert_eq!(first.queue_depth(), 1);
    assert_eq!(host.retained_bytes(), 60);

    let error = host
        .pump_with_budget(budget(4, 4))
        .expect_err("the second consumer must surface its callback panic");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::CallbackPanicked { .. }
    ));
    let receipts = host.fault_receipts();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].payload_json().is_none());
    assert!(receipts[0].payload_was_truncated());
    assert!(receipts[0].payload_digest().is_some());
    assert_eq!(pending_state.lock().unwrap().sequences, [1, 2]);
    assert_eq!(host.retained_bytes(), 0);
}
