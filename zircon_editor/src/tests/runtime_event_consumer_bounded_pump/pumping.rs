use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerPendingDeliveryBudget,
};

use super::{
    budget, register_state, FakeGateway, RecordingState, ReentrantObservationState, CAPABILITY,
};

#[test]
fn bounded_pump_defers_backlog_without_losing_order() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let state = Arc::new(Mutex::new(RecordingState {
        callback_delay: Duration::from_millis(1),
        ..RecordingState::default()
    }));
    register_state(&host, "tests.consumer.a", "tests.events.a", state.clone());
    host.begin_play_session(100, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in 1..=10 {
        gateway.push(11, "tests.events.a", sequence);
    }

    let first = host.pump_with_budget(budget(3, 3)).unwrap();
    assert_eq!(first.applied(), 3);
    assert_eq!(first.drained(), 10);
    assert_eq!(first.drained_encoded_bytes(), 110);
    assert_eq!(first.deferred(), 7);
    assert_eq!(first.queue_depth(), 7);
    assert_eq!(first.pending_encoded_bytes_upper_bound(), 77);
    assert!(!first.pending_oldest_age().is_zero());
    let first_runtime_backlog = first.runtime_backlog_observation();
    assert_eq!(
        first_runtime_backlog.known_remaining_deliveries_lower_bound(),
        0
    );
    assert_eq!(first_runtime_backlog.sampled_consumer_count(), 1);
    assert_eq!(first_runtime_backlog.unknown_consumer_count(), 0);
    assert_eq!(
        first_runtime_backlog.max_oldest_pending_age_millis(),
        Some(0)
    );
    assert!(first_runtime_backlog.max_observation_age().is_some());
    assert_eq!(gateway.drain_call_count(11), 1);
    assert_eq!(state.lock().unwrap().sequences, [1, 2, 3]);

    for sequence in 11..=12 {
        gateway.push(11, "tests.events.a", sequence);
    }
    let second = host.pump_with_budget(budget(3, 3)).unwrap();
    assert_eq!(second.applied(), 3);
    assert_eq!(second.drained(), 0);
    let second_runtime_backlog = second.runtime_backlog_observation();
    assert_eq!(
        second_runtime_backlog.known_remaining_deliveries_lower_bound(),
        0
    );
    assert_eq!(second_runtime_backlog.sampled_consumer_count(), 1);
    assert_eq!(second_runtime_backlog.unknown_consumer_count(), 0);
    assert_eq!(
        second_runtime_backlog.max_oldest_pending_age_millis(),
        Some(0)
    );
    assert!(second_runtime_backlog.max_observation_age().is_some());
    assert_eq!(gateway.drain_call_count(11), 1);
    assert_eq!(state.lock().unwrap().sequences, [1, 2, 3, 4, 5, 6]);

    while host.last_pump_report().queue_depth() != 0 {
        host.pump_with_budget(budget(3, 3)).unwrap();
    }
    assert_eq!(
        state.lock().unwrap().sequences,
        (1..=10).collect::<Vec<_>>()
    );

    let next_page = host.pump_with_budget(budget(3, 3)).unwrap();
    assert_eq!(next_page.applied(), 2);
    assert_eq!(next_page.drained(), 2);
    assert_eq!(gateway.drain_call_count(11), 2);
    assert_eq!(
        state.lock().unwrap().sequences,
        (1..=12).collect::<Vec<_>>()
    );
}

#[test]
fn total_pending_delivery_retention_rejects_an_over_budget_page() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host = EditorRuntimeEventConsumerHost::with_pending_delivery_budget(
        EditorRuntimeGatewayHandle::new(gateway.clone()),
        EditorRuntimeEventConsumerPendingDeliveryBudget::new(22),
    );
    let first = Arc::new(Mutex::new(RecordingState::default()));
    let second = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", first.clone());
    register_state(&host, "tests.consumer.b", "tests.events.b", second.clone());
    host.begin_play_session(150, &[CAPABILITY.to_string()])
        .unwrap();
    for sequence in 1..=2 {
        gateway.push(11, "tests.events.a", sequence);
        gateway.push(12, "tests.events.b", sequence);
    }

    let report = host.pump_with_budget(budget(2, 1)).unwrap();

    assert_eq!(report.applied(), 1);
    assert_eq!(report.dropped(), 2);
    assert_eq!(report.queue_depth(), 1);
    assert_eq!(report.pending_encoded_bytes_upper_bound(), 11);
    assert_eq!(first.lock().unwrap().sequences, [1]);
    assert!(second.lock().unwrap().sequences.is_empty());
}

#[test]
fn partial_runtime_backlog_observation_preserves_known_lower_bound() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    register_state(
        &host,
        "tests.consumer.a",
        "tests.events.a",
        Arc::new(Mutex::new(RecordingState::default())),
    );
    register_state(
        &host,
        "tests.consumer.b",
        "tests.events.b",
        Arc::new(Mutex::new(RecordingState::default())),
    );
    host.begin_play_session(100, &[CAPABILITY.to_string()])
        .unwrap();
    gateway.push(11, "tests.events.a", 1);
    gateway.set_runtime_backlog(11, 41, 17);

    let report = host.pump_with_budget(budget(1, 1)).unwrap();
    let observation = report.runtime_backlog_observation();

    assert_eq!(observation.known_remaining_deliveries_lower_bound(), 41);
    assert_eq!(observation.sampled_consumer_count(), 1);
    assert_eq!(observation.unknown_consumer_count(), 1);
    assert_eq!(observation.max_oldest_pending_age_millis(), Some(17));
    assert!(observation.max_observation_age().is_some());
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
