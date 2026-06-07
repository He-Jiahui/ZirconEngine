use super::fixture::{assert_absent, assert_contains, assert_ordered, EventBusSources};

#[test]
fn event_bus_failure_tracking_is_lazy_and_small_failed_lists_are_direct() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.failure,
        &[
            "fn record_failed_subscriber(",
            "if failed_topic.is_none()",
            "*failed_topic = Some(failed_event.topic);",
            "if let Some(failed_subscriber_list) = failed_subscribers.as_mut()",
            "failed_subscriber_list.push(subscriber.clone())",
            "} else if let Some(first_failed_subscriber) = first_failed_subscriber.take()",
            "Vec::with_capacity(subscriber_count)",
            "failed_subscriber_list.push(first_failed_subscriber)",
            "failed_subscriber_list.push(subscriber.clone())",
            "*failed_subscribers = Some(failed_subscriber_list)",
            "*first_failed_subscriber = Some(subscriber.clone())",
            "fn subscriber_failed(",
            "if let [failed_subscriber] = failed_subscribers",
            "return subscriber.same_channel(failed_subscriber);",
            "if let [first_failed_subscriber, second_failed_subscriber] = failed_subscribers",
            "subscriber.same_channel(first_failed_subscriber)",
            "|| subscriber.same_channel(second_failed_subscriber)",
            "if let [first_failed_subscriber, second_failed_subscriber, third_failed_subscriber]",
            "|| subscriber.same_channel(third_failed_subscriber)",
            "fourth_failed_subscriber",
            "|| subscriber.same_channel(fourth_failed_subscriber)",
            "fifth_failed_subscriber",
            "|| subscriber.same_channel(fifth_failed_subscriber)",
            ".iter()",
            ".any(|failed_subscriber| subscriber.same_channel(failed_subscriber))",
        ],
    );
    assert_contains(sources.publish, "fn prune_failed_publish_subscribers(");
    assert_contains(
        sources.publish,
        "self.prune_topic_subscribers(&topic, &failed_subscribers);",
    );
    assert_contains(
        sources.publish,
        "} else if let Some(failed_subscriber) = first_failed_subscriber",
    );
    assert_contains(sources.publish, "std::slice::from_ref(&failed_subscriber)");
    assert_absent(
        sources.failure,
        "failed_topic.get_or_insert(failed_event.topic);",
    );
}
