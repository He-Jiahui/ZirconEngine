use super::fixture::{assert_absent, assert_contains, EventBusSources};

#[test]
fn event_bus_root_stays_folder_backed_and_structural() {
    let sources = EventBusSources::load();

    assert_contains(sources.root, "mod failure;");
    assert_contains(sources.root, "mod prune;");
    assert_contains(sources.root, "mod publish;");
    assert_contains(sources.root, "mod subscribe;");
    assert_absent(sources.root, "pub fn subscribe(");
    assert_absent(sources.root, "pub fn publish(");
    assert_absent(sources.root, "fn prune_topic_subscribers(");
    assert_absent(sources.root, "fn subscriber_failed(");
    assert_absent(sources.root, "pub struct EngineEvent");
    assert_contains(
        sources.root,
        "use crate::core::framework::events::EngineEvent;",
    );
    assert_contains(sources.root, "pub struct EventBus");
    assert_contains(
        sources.root,
        "HashMap<String, Arc<[ChannelSender<EngineEvent>]>>",
    );
    assert_contains(sources.root, "delivery_lock: Arc<Mutex<()>>");
    assert_contains(sources.subscribe, "use std::collections::hash_map::Entry;");
    assert_contains(sources.publish, "use crossbeam_channel::SendError;");
    assert_contains(
        sources.combined.as_str(),
        "use crossbeam_channel::unbounded;",
    );
}
