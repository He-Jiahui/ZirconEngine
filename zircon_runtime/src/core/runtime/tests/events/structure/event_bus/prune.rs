use super::fixture::{EventBusSources, assert_absent, assert_contains, assert_ordered};

#[test]
fn event_bus_drop_deactivates_and_removes_the_subscription_from_its_topic() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.prune,
        &[
            "impl EventBusState",
            "pub(super) fn unsubscribe(",
            "let _delivery = topic.lock_delivery();",
            "subscriber.deactivate_and_drain();",
            "topic.remove_subscribers_while_delivery_locked(&[subscriber.id()])",
            "drop(_delivery);",
            "self.remove_topic_if_empty(topic);",
        ],
    );
    assert_contains(sources.subscriber, "impl Drop for EventSubscription");
    assert_contains(sources.subscriber, "state: Weak<EventBusState>");
    assert_contains(
        sources.subscriber,
        "if let Some(state) = self.state.upgrade()",
    );
    assert_contains(
        sources.subscriber,
        "state.unsubscribe(&self.topic, &self.subscriber);",
    );
    assert_contains(
        sources.subscriber,
        "self.subscriber.deactivate_and_drain();",
    );
    assert_contains(sources.topic, "impl Drop for EventBusState");
    assert_contains(sources.topic, "subscriber.deactivate_and_drain();");
    assert_contains(
        sources.topic,
        "pub(super) fn remove_subscribers_while_delivery_locked(",
    );
    assert_absent(sources.topic, "pub(super) fn remove_subscriber(");
    assert_absent(sources.prune, "ChannelSender<EngineEvent>");
    assert_absent(sources.prune, "prune_topic_subscribers");
}
