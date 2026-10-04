use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost, EditorRuntimeEventPumpBudget,
};

use super::support::{
    register_state, BlockingState, FakeGateway, RecordingState, ReentrantReconcileState, CAPABILITY,
};

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
