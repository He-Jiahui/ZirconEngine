pub(super) struct EventBusSources {
    pub(super) root: &'static str,
    pub(super) subscribe: &'static str,
    pub(super) publish: &'static str,
    pub(super) failure: &'static str,
    pub(super) prune: &'static str,
    pub(super) combined: String,
    pub(super) normalized_combined: String,
}

impl EventBusSources {
    pub(super) fn load() -> Self {
        let root = include_str!("../../../../../event_bus.rs");
        let subscribe = include_str!("../../../../../event_bus/subscribe.rs");
        let publish = include_str!("../../../../../event_bus/publish.rs");
        let failure = include_str!("../../../../../event_bus/failure.rs");
        let prune = include_str!("../../../../../event_bus/prune.rs");
        let combined = format!("{subscribe}\n{publish}\n{failure}\n{prune}\n{root}");
        let normalized_combined = combined.replace("\r\n", "\n");
        Self {
            root,
            subscribe,
            publish,
            failure,
            prune,
            combined,
            normalized_combined,
        }
    }

    pub(super) fn publish_body(&self) -> &str {
        slice_between(
            self.publish,
            "pub fn publish(&self, event: EngineEvent)",
            "fn snapshot_topic_subscribers(&self, topic: &str)",
        )
    }

    pub(super) fn occupied_subscribe_body(&self) -> &str {
        slice_between(self.subscribe, "Entry::Occupied(mut entry)", "        rx")
    }

    pub(super) fn prune_body(&self) -> &str {
        slice_from(self.prune, "fn prune_topic_subscribers(")
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

pub(super) fn slice_between<'a>(haystack: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = assert_contains(haystack, start);
    let end_index = haystack[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("expected source to contain end marker `{end}`"));
    &haystack[start_index..end_index]
}

pub(super) fn slice_from<'a>(haystack: &'a str, start: &str) -> &'a str {
    let start_index = assert_contains(haystack, start);
    &haystack[start_index..]
}
