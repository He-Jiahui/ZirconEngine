use super::fixture::{
    assert_absent, assert_contains, assert_ordered, slice_between, EventBusSources,
};

#[test]
fn event_bus_prune_dispatches_small_topic_lists_before_lazy_retained_rebuild() {
    let sources = EventBusSources::load();
    let prune_body = sources.prune_body();
    let three_body = slice_between(
        sources.prune,
        "[first_subscriber, second_subscriber, third_subscriber] =>",
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
    );
    let four_body = slice_between(
        sources.prune,
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber] =>",
    );
    let five_body = slice_between(
        sources.prune,
        "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber] =>",
        "current_subscribers =>",
    );

    assert_ordered(
        prune_body,
        &[
            "match topic_subscribers.as_ref()",
            "[] => true,",
            "[subscriber] => Self::subscriber_failed(subscriber, failed_subscribers),",
            "[first_subscriber, second_subscriber] =>",
            "let first_failed =",
            "Self::subscriber_failed(first_subscriber, failed_subscribers);",
            "let second_failed =",
            "Self::subscriber_failed(second_subscriber, failed_subscribers);",
            "match (first_failed, second_failed)",
            "(true, true) => true,",
            "second_subscriber.clone()",
            "first_subscriber.clone()",
            "[first_subscriber, second_subscriber, third_subscriber] =>",
            "let first_failed =",
            "Self::subscriber_failed(first_subscriber, failed_subscribers);",
            "let second_failed =",
            "Self::subscriber_failed(second_subscriber, failed_subscribers);",
            "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
            "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber] =>",
            "current_subscribers =>",
            "let mut retained_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;",
            "let mut saw_failed_subscriber = false;",
            "for (subscriber_index, subscriber) in current_subscribers.iter().enumerate()",
            "if Self::subscriber_failed(subscriber, failed_subscribers)",
            "!saw_failed_subscriber",
            "subscriber_index > 0",
            "current_subscribers[..subscriber_index].iter().cloned()",
            "saw_failed_subscriber = true;",
            "if saw_failed_subscriber",
            ".get_or_insert_with(||",
            "Vec::with_capacity(current_subscribers.len())",
            ".push(subscriber.clone())",
            "if let Some(retained_subscribers) = retained_subscribers",
            "*topic_subscribers = retained_subscribers.into();",
            "saw_failed_subscriber",
        ],
    );
    assert_ordered(
        three_body,
        &[
            "let third_failed =",
            "match (first_failed, second_failed, third_failed)",
            "(true, true, true) => true,",
            "(true, true, false) =>",
            "third_subscriber.clone()",
            "(true, false, true) =>",
            "second_subscriber.clone()",
            "(false, true, true) =>",
            "first_subscriber.clone()",
            "(true, false, false) =>",
            "(false, true, false) =>",
            "(false, false, true) =>",
            "(false, false, false) => false,",
        ],
    );
    assert_ordered(
        four_body,
        &[
            "let fourth_failed =",
            "match (first_failed, second_failed, third_failed, fourth_failed)",
            "(true, true, true, true) => true,",
            "(true, true, true, false) =>",
            "fourth_subscriber.clone()",
            "(true, true, false, true) =>",
            "third_subscriber.clone()",
            "(true, false, true, true) =>",
            "second_subscriber.clone()",
            "(false, true, true, true) =>",
            "first_subscriber.clone()",
            "(true, false, false, false) =>",
            "(false, false, false, true) =>",
            "(false, false, false, false) => false,",
        ],
    );
    assert_ordered(
        five_body,
        &[
            "let fifth_failed =",
            "let failed_tuple =",
            "(true, true, true, true, true) => true,",
            "(false, false, false, false, false) => false,",
            "Self::exact_five_surviving_subscribers(",
            "fifth_subscriber,",
            "*topic_subscribers = surviving_subscribers;",
        ],
    );
    assert_contains(sources.prune, "fn exact_five_surviving_subscribers(");
    assert_contains(sources.prune, "fn record_surviving_subscriber(");
    assert_contains(sources.prune, "fn surviving_subscriber_slice(");
    assert_contains(sources.prune, "let mut first_survivor = None;");
    assert_contains(sources.prune, "let mut fifth_survivor = None;");
    assert_contains(sources.prune, "match survivor_count");
    assert_contains(
        sources.prune,
        "1 => Arc::<[ChannelSender<EngineEvent>]>::from([",
    );
    assert_contains(
        sources.prune,
        "4 => Arc::<[ChannelSender<EngineEvent>]>::from([",
    );
    assert_absent(five_body, "Vec::with_capacity");
    assert_absent(five_body, "retained_subscribers");
    assert_absent(prune_body, "for subscriber in current_subscribers");
    assert_absent(prune_body, "for subscriber in topic_subscribers.iter()");
    assert_absent(prune_body, "Vec::with_capacity(2)");
    assert_absent(prune_body, "Vec::with_capacity(topic_subscribers.len())");
    assert_absent(
        prune_body,
        "let is_empty = retained_subscribers.is_empty();",
    );
    assert_absent(
        prune_body,
        ".cloned()\n                .collect::<Vec<_>>()",
    );
}
