use std::sync::{Arc, Mutex};

use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerHost;

use super::{budget, register_state, FakeGateway, RecordingState, CAPABILITY};

#[test]
fn global_budget_resumes_at_the_first_unvisited_consumer() {
    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let first = Arc::new(Mutex::new(RecordingState::default()));
    let second = Arc::new(Mutex::new(RecordingState::default()));
    let third = Arc::new(Mutex::new(RecordingState::default()));
    let fourth = Arc::new(Mutex::new(RecordingState::default()));
    register_state(&host, "tests.consumer.a", "tests.events.a", first.clone());
    register_state(&host, "tests.consumer.b", "tests.events.b", second.clone());
    register_state(&host, "tests.consumer.c", "tests.events.c", third.clone());
    register_state(&host, "tests.consumer.d", "tests.events.d", fourth.clone());
    host.begin_play_session(230, &[CAPABILITY.to_string()])
        .expect("all test consumers should start");

    for sequence in 1..=3 {
        gateway.push(11, "tests.events.a", sequence);
        gateway.push(12, "tests.events.b", sequence);
        gateway.push(13, "tests.events.c", sequence);
        gateway.push(14, "tests.events.d", sequence);
    }

    host.pump_with_budget(budget(3, 1))
        .expect("first global budget should apply three consumers");
    assert_eq!(first.lock().unwrap().sequences, [1]);
    assert_eq!(second.lock().unwrap().sequences, [1]);
    assert_eq!(third.lock().unwrap().sequences, [1]);
    assert!(fourth.lock().unwrap().sequences.is_empty());

    let zero_budget = host
        .pump_with_budget(budget(0, 1))
        .expect("a zero-budget frame should preserve the current round-robin cursor");
    assert_eq!(zero_budget.applied(), 0);

    host.pump_with_budget(budget(3, 1))
        .expect("second global budget should resume with the unvisited consumer");
    assert_eq!(first.lock().unwrap().sequences, [1, 2]);
    assert_eq!(second.lock().unwrap().sequences, [1, 2]);
    assert_eq!(third.lock().unwrap().sequences, [1]);
    assert_eq!(fourth.lock().unwrap().sequences, [1]);
}

#[test]
fn global_budget_covers_sixty_four_consumers_without_revisiting_the_prefix() {
    const CONSUMER_COUNT: usize = 64;

    let gateway = Arc::new(FakeGateway::new(7));
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway.clone()));
    let states = (0..CONSUMER_COUNT)
        .map(|index| {
            let state = Arc::new(Mutex::new(RecordingState::default()));
            register_state(
                &host,
                &format!("tests.consumer.fair-{index:02}"),
                &format!("tests.events.fair-{index:02}"),
                state.clone(),
            );
            state
        })
        .collect::<Vec<_>>();
    host.begin_play_session(231, &[CAPABILITY.to_string()])
        .expect("all fairness consumers should start");

    for index in 0..CONSUMER_COUNT {
        let subscription = 11 + u64::try_from(index).expect("fixture index fits subscription");
        gateway.push(subscription, &format!("tests.events.fair-{index:02}"), 1);
    }

    for _ in 0..(CONSUMER_COUNT / 8) {
        host.pump_with_budget(budget(8, 1))
            .expect("each bounded frame should apply a distinct consumer window");
    }

    for state in states {
        assert_eq!(state.lock().unwrap().sequences, [1]);
    }
}
