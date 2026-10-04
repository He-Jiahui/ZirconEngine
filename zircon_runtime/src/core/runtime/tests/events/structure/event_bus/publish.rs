use super::fixture::{assert_absent, assert_contains, assert_ordered, EventBusSources};

#[test]
fn event_bus_publish_shares_one_immutable_payload_under_a_per_topic_delivery_lock() {
    let sources = EventBusSources::load();
    assert_contains(sources.publish, "pub fn try_publish(");
    assert_contains(
        sources.publish,
        "Result<EngineEventPublishReceipt, EngineEventPublishRejected>",
    );
    assert_contains(
        sources.publish,
        "EngineEventPublishRejection::NoSubscribers",
    );
    assert_contains(sources.publish, "topic.lock_delivery()");
    assert_contains(sources.publish, "super::frozen::freeze(event, limits)");
    assert_contains(
        sources.publish,
        "EngineEventPublishRejected { reason, event }",
    );
    assert_absent(sources.publish, "pub fn publish(");
    assert_absent(sources.publish, "subscriber.deliver(");
}

#[test]
fn event_subscriber_linearizes_physical_queue_changes_with_depth_accounting() {
    let sources = EventBusSources::load();
    assert_contains(sources.subscriber, "queue: VecDeque<QueuedEngineEvent>");
    assert_contains(sources.subscriber, "queue_ready: Condvar");
    assert_ordered(
        sources.publish,
        &[
            "drop(ledger);",
            "queues.clear();",
            "drop(displaced);",
            "drop(_commit);",
            "drop(_delivery);",
        ],
    );
    assert_contains(sources.subscriber, "std::mem::take(&mut queue_state.queue)");
    assert_contains(sources.subscriber, "record_dequeued_depth()");
    assert_absent(sources.subscriber, "crossbeam_channel");
}

#[test]
fn event_bus_diagnostics_sample_routine_timings_and_measure_every_delivery_wait() {
    let sources = EventBusSources::load();
    assert_contains(sources.diagnostics, "routine_timing_sample_interval: u64");
    assert_contains(sources.diagnostics, "waiting_publishers: AtomicU64");
    assert_contains(sources.publish, "record_publisher_waiting()");
    assert_contains(sources.publish, "record_publisher_resumed(wait_started)");
    assert_contains(sources.topic, "Err(TryLockError::WouldBlock) => None");
    assert_contains(sources.publish, "self.admission.lock()");
}
