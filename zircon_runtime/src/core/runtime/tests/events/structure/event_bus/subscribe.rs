use super::fixture::{EventBusSources, assert_absent, assert_contains, assert_ordered};

#[test]
fn event_bus_subscribe_binds_an_explicit_delivery_policy_to_state() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.subscribe,
        &[
            "use crate::core::framework::events::{",
            "EngineEventDeliveryPolicy,",
            "EngineEventSubscription};",
            "impl EventBus",
            "pub fn subscribe(",
            "policy: EngineEventDeliveryPolicy",
            "Box<dyn EngineEventSubscription>",
            "Box::new(self.state.subscribe(topic.into(), policy))",
        ],
    );
    assert_contains(sources.topic, "pub(super) fn subscribe(");
    assert_contains(sources.topic, "EventSubscriber::new(");
    assert_ordered(
        sources.topic,
        &[
            "let (topic, reservation) = {",
            "let mut topics = self.lock_topics();",
            "let reservation = topic.reserve_subscription();",
            "(topic, reservation)",
            "after_reservation();",
            "topic.add_subscriber",
            "drop(reservation);",
            "EventSubscription::new",
        ],
    );
    assert_contains(sources.topic, "pending_subscriptions: AtomicUsize");
    assert_contains(sources.topic, "struct PendingSubscription");
    assert_contains(sources.topic, "impl Drop for PendingSubscription");
    assert_contains(sources.topic, "topic.is_removable()");
    assert_absent(sources.subscribe, "ChannelReceiver<EngineEvent>");
    assert_absent(sources.subscribe, "Entry::Occupied");
}
