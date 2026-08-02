use super::fixture::{assert_absent, assert_contains, EventBusSources};

#[test]
fn event_bus_root_stays_folder_backed_and_structural() {
    let sources = EventBusSources::load();

    assert_contains(sources.root, "mod diagnostics;");
    assert_contains(sources.root, "mod prune;");
    assert_contains(sources.root, "mod publish;");
    assert_contains(sources.root, "mod subscribe;");
    assert_contains(sources.root, "mod subscriber;");
    assert_contains(sources.root, "mod topic;");
    assert_absent(sources.root, "pub fn subscribe(");
    assert_absent(sources.root, "pub fn publish(");
    assert_absent(sources.root, "EventSubscriberMap");
    assert_absent(sources.root, "delivery_lock:");
    assert_absent(sources.root, "lock_subscribers");
    assert_absent(sources.root, "lock_delivery");
    assert_absent(sources.root, "pub struct EngineEvent");
    assert_contains(sources.root, "use topic::EventBusState;");
    assert_contains(sources.root, "pub struct EventBus");
    assert_contains(sources.root, "state: Arc<EventBusState>");
    assert_contains(sources.topic, "pub(super) struct EventBusState");
    assert_contains(sources.subscriber, "pub(super) struct EventSubscription");
    assert_contains(
        sources.diagnostics,
        "pub(super) struct EventBusDiagnosticsState",
    );
    assert_contains(
        sources.combined.as_str(),
        "unwrap_or_else(|poisoned| poisoned.into_inner())",
    );
}
