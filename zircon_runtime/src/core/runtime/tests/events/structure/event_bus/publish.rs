use super::fixture::{assert_absent, assert_contains, assert_ordered, EventBusSources};

#[test]
fn event_bus_publish_shares_one_immutable_payload_under_a_per_topic_delivery_lock() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.publish,
        &[
            "impl EventBus",
            "pub fn publish(&self, event: EngineEvent)",
            "self.state.publish(event);",
            "pub fn diagnostic_report(&self)",
            "self.state.diagnostic_report()",
            "impl EventBusState",
            "pub(super) fn publish(&self, event: EngineEvent)",
            "self.diagnostics.record_published_and_capture_time();",
            "Arc::new(event)",
            "let _delivery = if let Some(delivery) = topic.try_lock_delivery()",
            "topic.snapshot_subscribers()",
            "subscribers.split_last()",
            "subscriber.deliver(Arc::clone(&event))",
            "last_subscriber.deliver(event)",
            "EventDeliveryStatus::Disconnected",
            "topic.remove_subscribers_while_delivery_locked",
            "self.remove_topic_if_empty(&topic);",
            "self.diagnostics.record_publish_duration(started);",
        ],
    );
    assert_contains(sources.subscriber, "event: Arc<EngineEvent>");
    assert_absent(sources.publish, "ChannelSender<EngineEvent>");
    assert_absent(sources.publish, "lock_subscribers()");
    assert_absent(sources.publish, "event.clone()");
    assert_absent(
        sources.publish,
        "last_subscriber.deliver(Arc::clone(&event))",
    );
}

#[test]
fn event_subscriber_linearizes_physical_queue_changes_with_depth_accounting() {
    let sources = EventBusSources::load();

    assert_contains(sources.subscriber, "queue: VecDeque<QueuedEngineEvent>");
    assert_contains(sources.subscriber, "queue_ready: Condvar");
    assert_ordered(
        sources.subscriber,
        &[
            "let mut queue_state = self.lock_queue_state();",
            "let dropped = capacity",
            ".is_some_and",
            ".pop_front()",
            "queue_state.queue.push_back",
            "record_replaced_and_capture_time",
            "self.diagnostics.record_enqueued_and_capture_time()",
            "drop(queue_state);",
            "self.queue_ready.notify_one();",
            "fn pop_front_while_locked",
            "queue_state.queue.pop_front()",
            "queued.queued_at.map(|queued_at| queued_at.elapsed())",
            "self.diagnostics.record_dequeued_depth()",
            "fn finalize_dequeued_event",
            "self.diagnostics.record_dequeued_age(dequeued.queue_age)",
            "pub(super) fn deactivate_and_drain",
            "std::mem::take(&mut queue_state.queue)",
            "for queued in queued",
            "self.diagnostics.record_dequeued",
        ],
    );
    assert_absent(sources.subscriber, "crossbeam_channel");
    assert_absent(sources.subscriber, ".recv()");
    assert_absent(sources.subscriber, ".try_recv()");
    assert_absent(sources.diagnostics, "fn record_overflow_drop");
}

#[test]
fn event_bus_diagnostics_sample_routine_timings_and_measure_every_delivery_wait() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.publish,
        &[
            "let started = self.diagnostics.record_published_and_capture_time();",
            "let _delivery = if let Some(delivery) = topic.try_lock_delivery()",
            "delivery",
            "} else {",
            "let wait_started = self.diagnostics.capture_contention_time();",
            "self.diagnostics.record_publisher_waiting();",
            "let delivery = topic.lock_delivery();",
            "self.diagnostics.record_publisher_resumed(wait_started);",
            "self.diagnostics.record_publish_duration(started);",
        ],
    );
    assert_contains(sources.diagnostics, "enabled: bool");
    assert_contains(sources.diagnostics, "routine_timing_sample_interval: u64");
    assert_contains(sources.diagnostics, "self.enabled.then(Instant::now)");
    assert_contains(
        sources.diagnostics,
        "sample_index % self.routine_timing_sample_interval == 0",
    );
    assert_contains(sources.diagnostics, "if !self.enabled");
    assert_contains(sources.diagnostics, "waiting_publishers: AtomicU64");
    assert_contains(sources.diagnostics, "delivery_lock_wait_samples: AtomicU64");
    assert_contains(sources.topic, "pub(super) fn try_lock_delivery(");
    assert_contains(sources.topic, "Err(TryLockError::WouldBlock) => None");
    assert_contains(
        sources.topic,
        "Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner())",
    );
    assert_ordered(
        sources.diagnostics,
        &[
            "pub(super) fn snapshot(",
            "if !self.enabled",
            "return EventBusDiagnosticsSnapshot",
            "..EventBusDiagnosticsSnapshot::default()",
            "let queued = self.queued.load(Ordering::Acquire);",
        ],
    );
    assert_contains(sources.subscriber, "queued_at: None");
    assert_contains(
        sources.subscriber,
        "self.diagnostics.record_enqueued_and_capture_time()",
    );
    assert_contains(sources.diagnostics, "fn record_dequeued_depth");
    assert_contains(sources.diagnostics, "fn record_dequeued_age");
    assert_absent(sources.publish, "Instant::now()");
}
