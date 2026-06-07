use super::fixture::{assert_absent, assert_contains, assert_ordered, EventBusSources};

#[test]
fn event_bus_subscribe_uses_arc_slice_fast_paths_before_vec_append() {
    let sources = EventBusSources::load();
    let occupied_body = sources.occupied_subscribe_body();

    assert_ordered(
        sources.subscribe,
        &[
            "Entry::Vacant(entry)",
            "entry.insert(Arc::<[ChannelSender<EngineEvent>]>::from([tx]))",
            "Entry::Occupied(mut entry)",
            "match topic_subscribers.as_ref()",
        ],
    );
    assert_ordered(
        occupied_body,
        &[
            "[] => Arc::<[ChannelSender<EngineEvent>]>::from([tx])",
            "[subscriber] =>",
            "Arc::<[ChannelSender<EngineEvent>]>::from([subscriber.clone(), tx])",
            "[first_subscriber, second_subscriber] =>",
            "first_subscriber.clone()",
            "second_subscriber.clone()",
            "tx,",
            "[first_subscriber, second_subscriber, third_subscriber] =>",
            "third_subscriber.clone()",
            "tx,",
            "[first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] =>",
            "fourth_subscriber.clone()",
            "tx,",
            "current_subscribers =>",
            "Vec::with_capacity(current_subscribers.len() + 1)",
            "updated_subscribers.extend(current_subscribers.iter().cloned())",
            "updated_subscribers.push(tx)",
            "*topic_subscribers = updated_subscribers",
        ],
    );
    assert_contains(occupied_body, "Arc::<[ChannelSender<EngineEvent>]>::from([");
    assert_absent(sources.subscribe, ".or_default()");
    assert_absent(sources.subscribe, "if topic_subscribers.is_empty()");
    assert_absent(
        occupied_body,
        "Vec::with_capacity(topic_subscribers.len() + 1)",
    );
    assert_absent(
        occupied_body,
        "updated_subscribers.extend(topic_subscribers.iter().cloned())",
    );
}
