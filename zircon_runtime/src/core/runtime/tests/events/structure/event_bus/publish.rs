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
            "self.diagnostics.record_published();",
            "Arc::new(event)",
            "let _delivery = if let Some(delivery) = topic.try_lock_delivery()",
            "topic.snapshot_subscribers()",
            "subscriber.deliver(Arc::clone(&event))",
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
            "let dropped = queue_state",
            ".pop_front()",
            "record_overflow_drop",
            "queue_state.queue.push_back",
            "self.diagnostics.record_enqueued();",
            "self.queue_ready.notify_one();",
            "fn pop_front_while_locked",
            "let queued = queue_state.queue.pop_front()?;",
            "self.diagnostics.record_dequeued",
        ],
    );
    assert_absent(sources.subscriber, "crossbeam_channel");
    assert_absent(sources.subscriber, ".recv()");
    assert_absent(sources.subscriber, ".try_recv()");
}

#[test]
fn event_bus_diagnostics_measure_delivery_wait_and_skip_timestamps_when_disabled() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.publish,
        &[
            "let started = self.diagnostics.capture_time();",
            "self.diagnostics.record_published();",
            "let _delivery = if let Some(delivery) = topic.try_lock_delivery()",
            "delivery",
            "} else {",
            "let wait_started = self.diagnostics.capture_time();",
            "self.diagnostics.record_publisher_waiting();",
            "let delivery = topic.lock_delivery();",
            "self.diagnostics.record_publisher_resumed(wait_started);",
            "self.diagnostics.record_publish_duration(started);",
        ],
    );
    assert_contains(sources.diagnostics, "enabled: bool");
    assert_contains(sources.diagnostics, "self.enabled.then(Instant::now)");
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
    assert_contains(
        sources.subscriber,
        "queued_at: self.diagnostics.capture_time()",
    );
    assert_absent(sources.publish, "Instant::now()");
}
