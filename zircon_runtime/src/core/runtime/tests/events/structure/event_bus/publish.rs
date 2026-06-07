use super::fixture::{
    assert_absent, assert_contains, assert_ordered, slice_between, EventBusSources,
};

#[test]
fn event_bus_publish_snapshots_before_delivery_and_moves_final_event() {
    let sources = EventBusSources::load();
    let publish_body = sources.publish_body();
    let three_body = slice_between(
        sources.publish,
        "[first_subscriber, second_subscriber, third_subscriber] =>",
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
    );
    let four_body = slice_between(
        sources.publish,
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber]",
    );
    let five_body = slice_between(
        sources.publish,
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber]",
        "[leading_subscribers @ .., last_subscriber]",
    );

    assert_ordered(
        publish_body,
        &[
            "let Some(subscribers) = self.snapshot_topic_subscribers(&event.topic) else",
            "let _delivery_guard = self.delivery_lock.lock().unwrap();",
            "match subscribers.as_ref()",
            "[] => return,",
            "[subscriber] =>",
            "subscriber.send(event)",
            "std::slice::from_ref(subscriber)",
            "[first_subscriber, second_subscriber] =>",
            "let subscriber_count = 2;",
            "first_subscriber.send(event.clone())",
            "second_subscriber.send(event)",
            "self.prune_failed_publish_subscribers(",
            "[first_subscriber, second_subscriber, third_subscriber] =>",
            "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
            "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber]",
            "[leading_subscribers @ .., last_subscriber]",
            "let subscriber_count = leading_subscribers.len() + 1;",
            "for subscriber in leading_subscribers",
            "subscriber.send(event.clone())",
            "last_subscriber.send(event)",
        ],
    );
    assert_ordered(
        three_body,
        &[
            "let subscriber_count = 3;",
            "first_subscriber.send(event.clone())",
            "second_subscriber.send(event.clone())",
            "third_subscriber.send(event)",
            "self.prune_failed_publish_subscribers(",
        ],
    );
    assert_ordered(
        four_body,
        &[
            "let subscriber_count = 4;",
            "first_subscriber.send(event.clone())",
            "second_subscriber.send(event.clone())",
            "third_subscriber.send(event.clone())",
            "fourth_subscriber.send(event)",
            "self.prune_failed_publish_subscribers(",
        ],
    );
    assert_ordered(
        five_body,
        &[
            "let subscriber_count = 5;",
            "first_subscriber.send(event.clone())",
            "second_subscriber.send(event.clone())",
            "third_subscriber.send(event.clone())",
            "fourth_subscriber.send(event.clone())",
            "fifth_subscriber.send(event)",
            "self.prune_failed_publish_subscribers(",
        ],
    );
    assert_contains(
        sources.publish,
        "fn snapshot_topic_subscribers(&self, topic: &str) -> Option<Arc<[ChannelSender<EngineEvent>]>>",
    );
    assert_contains(sources.publish, "let snapshot = subscribers.get(topic)?;");
    assert_contains(sources.publish, "if snapshot.is_empty()");
    assert_contains(sources.publish, "Some(snapshot.clone())");
    assert_contains(
        sources.normalized_combined.as_str(),
        "let snapshot = subscribers.get(topic)?;\n        if snapshot.is_empty()",
    );
    assert_absent(publish_body, "self.subscribers.lock()");
    assert_absent(publish_body, "if subscribers.is_empty()");
    assert_absent(publish_body, "let topic = event.topic.clone();");
    assert_absent(publish_body, "if let [subscriber] = subscribers.as_ref()");
    assert_absent(publish_body, "subscribers.split_last()");
    assert_absent(
        publish_body,
        "for (index, subscriber) in subscribers.iter().enumerate()",
    );
    assert_absent(publish_body, "if index + 1 == subscribers.len()");
    assert_absent(publish_body, "let mut event = Some(event);");
    assert_absent(publish_body, "event.take().unwrap()");
    assert_absent(publish_body, "event.as_ref().unwrap().clone()");
    assert_absent(publish_body, "let mut failed_subscribers = Vec::new();");
    assert_absent(
        publish_body,
        "let mut failed_subscribers = Vec::with_capacity",
    );
    assert_absent(publish_body, "collect::<Vec<_>>()");
    assert_absent(sources.publish, "unwrap_or_default()");
}
