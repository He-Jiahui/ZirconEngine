use std::collections::BTreeMap;

use woc_client::{
    armory_card_visible, build_welcome_screen_view, chest_tile_visible, consume_armory_open_intent,
    continue_button_state, discord_strip_visible, mark_new_releases, next_last_seen_release_id,
    set_armory_open_intent, welcome_continue_hint, ContinueButtonState, WelcomeChestInput,
    WelcomeConnectionInput, WelcomeContinueHint, WelcomeDiscordInput, WelcomeNewsInput,
    WelcomeNewsState, WelcomePlatformInput, WelcomeRelease, WelcomeReleaseSummary,
    WelcomeSessionStorage, MAX_WELCOME_RELEASES_SHOWN,
};

const DESKTOP_WEB: WelcomePlatformInput = WelcomePlatformInput {
    native_app: false,
    desktop_app: false,
    mobile_touch: false,
    offline: false,
};

#[test]
fn armory_and_chest_require_desktop_web_online_gates() {
    let ready_chest = WelcomeChestInput {
        ready: true,
        unknown: false,
    };
    assert!(armory_card_visible(DESKTOP_WEB, true));
    assert!(!armory_card_visible(DESKTOP_WEB, false));
    assert!(chest_tile_visible(DESKTOP_WEB, ready_chest));

    for platform in [
        WelcomePlatformInput {
            desktop_app: true,
            ..DESKTOP_WEB
        },
        WelcomePlatformInput {
            mobile_touch: true,
            ..DESKTOP_WEB
        },
        WelcomePlatformInput {
            native_app: true,
            ..DESKTOP_WEB
        },
        WelcomePlatformInput {
            offline: true,
            ..DESKTOP_WEB
        },
    ] {
        assert!(!armory_card_visible(platform, true));
        assert!(!chest_tile_visible(platform, ready_chest));
    }

    assert!(!chest_tile_visible(
        DESKTOP_WEB,
        WelcomeChestInput {
            ready: false,
            unknown: false,
        }
    ));
    assert!(!chest_tile_visible(
        DESKTOP_WEB,
        WelcomeChestInput {
            ready: true,
            unknown: true,
        }
    ));
}

#[test]
fn continue_waits_online_but_is_always_ready_offline() {
    assert_eq!(
        continue_button_state(WelcomeConnectionInput {
            ready: false,
            offline: false,
        }),
        ContinueButtonState::Connecting
    );
    assert_eq!(
        continue_button_state(WelcomeConnectionInput {
            ready: true,
            offline: false,
        }),
        ContinueButtonState::Ready
    );
    assert_eq!(
        continue_button_state(WelcomeConnectionInput {
            ready: false,
            offline: true,
        }),
        ContinueButtonState::Ready
    );
}

#[test]
fn discord_strip_matches_the_fail_open_matrix() {
    let base = WelcomeDiscordInput {
        enabled: Some(true),
        linked: Some(false),
        guild_member: Some(false),
        fetch_failed: false,
    };
    assert!(discord_strip_visible(base, false));
    assert!(!discord_strip_visible(
        WelcomeDiscordInput {
            linked: Some(true),
            guild_member: Some(true),
            ..base
        },
        false
    ));
    assert!(!discord_strip_visible(
        WelcomeDiscordInput {
            enabled: Some(false),
            ..base
        },
        false
    ));
    assert!(discord_strip_visible(
        WelcomeDiscordInput {
            enabled: None,
            linked: None,
            guild_member: None,
            fetch_failed: true,
        },
        false
    ));
    assert!(discord_strip_visible(
        WelcomeDiscordInput {
            linked: Some(true),
            guild_member: Some(true),
            ..base
        },
        true
    ));
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DetailedRelease {
    id: u64,
    tag: &'static str,
    body: &'static str,
}

impl WelcomeRelease for DetailedRelease {
    fn release_id(&self) -> u64 {
        self.id
    }
}

#[test]
fn release_marking_caps_at_five_and_preserves_superset_fields() {
    let releases = (1..=8)
        .rev()
        .map(|id| DetailedRelease {
            id,
            tag: "vNext",
            body: "full body",
        })
        .collect::<Vec<_>>();

    let marked = mark_new_releases(&releases, Some(6));
    assert_eq!(marked.len(), MAX_WELCOME_RELEASES_SHOWN);
    assert_eq!(marked[0].release.id, 8);
    assert_eq!(marked[0].release.body, "full body");
    assert!(marked[0].is_new);
    assert!(!marked[2].is_new);

    assert_eq!(next_last_seen_release_id(&releases, None), Some(8));
    assert_eq!(next_last_seen_release_id(&releases, Some(10)), Some(10));
    assert_eq!(
        next_last_seen_release_id::<DetailedRelease>(&[], Some(3)),
        Some(3)
    );
}

#[test]
fn first_visit_marks_every_shown_release_new() {
    let releases = vec![WelcomeReleaseSummary {
        id: 5,
        tag: "v0.26.0".to_string(),
        name: "v0.26.0".to_string(),
        published_at: "2026-07-10T00:00:00Z".to_string(),
    }];

    assert!(mark_new_releases(&releases, None)[0].is_new);
}

#[test]
fn composed_view_keeps_feed_failure_independent_from_other_tiles() {
    let view = build_welcome_screen_view(
        DESKTOP_WEB,
        &WelcomeNewsInput::<WelcomeReleaseSummary> {
            state: WelcomeNewsState::Failed,
            releases: Vec::new(),
        },
        WelcomeDiscordInput {
            enabled: Some(true),
            linked: Some(false),
            guild_member: Some(false),
            fetch_failed: false,
        },
        WelcomeChestInput {
            ready: false,
            unknown: true,
        },
        WelcomeConnectionInput {
            ready: true,
            offline: false,
        },
        true,
        None,
    );

    assert_eq!(view.news_state, WelcomeNewsState::Failed);
    assert!(view.releases.is_empty());
    assert!(view.show_armory_card);
    assert!(view.show_discord_strip);
    assert!(!view.show_chest_tile);
    assert_eq!(view.continue_state, ContinueButtonState::Ready);
}

#[derive(Default)]
struct FakeSessionStorage {
    values: BTreeMap<String, String>,
}

impl WelcomeSessionStorage for FakeSessionStorage {
    fn get_item(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    fn set_item(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    fn remove_item(&mut self, key: &str) {
        self.values.remove(key);
    }
}

#[test]
fn armory_intent_is_absent_then_consumed_exactly_once() {
    let mut storage = FakeSessionStorage::default();
    assert!(!consume_armory_open_intent(&mut storage));

    set_armory_open_intent(&mut storage);
    assert!(consume_armory_open_intent(&mut storage));
    assert!(!consume_armory_open_intent(&mut storage));
}

#[test]
fn native_and_touch_hosts_use_the_touch_continue_hint() {
    assert_eq!(
        welcome_continue_hint(DESKTOP_WEB),
        WelcomeContinueHint::Keyboard
    );
    assert_eq!(
        welcome_continue_hint(WelcomePlatformInput {
            native_app: true,
            ..DESKTOP_WEB
        }),
        WelcomeContinueHint::Touch
    );
    assert_eq!(
        welcome_continue_hint(WelcomePlatformInput {
            mobile_touch: true,
            ..DESKTOP_WEB
        }),
        WelcomeContinueHint::Touch
    );
}
