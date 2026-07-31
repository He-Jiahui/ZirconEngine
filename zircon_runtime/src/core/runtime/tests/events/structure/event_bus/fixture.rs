pub(super) struct EventBusSources {
    pub(super) root: &'static str,
    pub(super) subscribe: &'static str,
    pub(super) publish: &'static str,
    pub(super) prune: &'static str,
    pub(super) diagnostics: &'static str,
    pub(super) subscriber: &'static str,
    pub(super) topic: &'static str,
    pub(super) combined: String,
}

impl EventBusSources {
    pub(super) fn load() -> Self {
        let root = include_str!("../../../../events.rs");
        let subscribe = include_str!("../../../../events/subscribe.rs");
        let publish = include_str!("../../../../events/publish.rs");
        let prune = include_str!("../../../../events/prune.rs");
        let diagnostics = include_str!("../../../../events/diagnostics.rs");
        let subscriber = include_str!("../../../../events/subscriber.rs");
        let topic = include_str!("../../../../events/topic.rs");
        let combined = format!(
            "{subscribe}\n{publish}\n{prune}\n{diagnostics}\n{subscriber}\n{topic}\n{root}"
        );
        Self {
            root,
            subscribe,
            publish,
            prune,
            diagnostics,
            subscriber,
            topic,
            combined,
        }
    }
}

pub(super) fn assert_contains(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("expected source to contain `{needle}`"))
}

pub(super) fn assert_absent(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "expected source not to contain `{needle}`"
    );
}

pub(super) fn assert_ordered(haystack: &str, fragments: &[&str]) {
    let mut cursor = 0;
    for fragment in fragments {
        let offset = haystack[cursor..]
            .find(fragment)
            .unwrap_or_else(|| panic!("expected ordered fragment `{fragment}`"));
        cursor += offset + fragment.len();
    }
}
