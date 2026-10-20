use crate::asset::AssetUri;

use super::{AssetChangeKind, AssetWatchEvent, AssetWatcher};

#[test]
fn runtime88_borrowed_event_fold_batch_repeated_modifications_clone_only_unique_result_uri() {
    let uri = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    let events = (0..4_096)
        .map(|_| AssetWatchEvent::Modified(uri.clone()))
        .collect::<Vec<_>>();

    let changes = AssetWatcher::fold_events(&events);

    assert_eq!(events.len(), 4_096);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, AssetChangeKind::Modified);
    assert_eq!(changes[0].uri, uri);
    assert_eq!(changes[0].previous_uri, None);
}

#[test]
fn runtime88_borrowed_event_fold_batch_added_remains_added_after_repeated_modifications() {
    let uri = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    let mut events = vec![AssetWatchEvent::Added(uri.clone())];
    events.extend((0..4_096).map(|_| AssetWatchEvent::Modified(uri.clone())));

    let changes = AssetWatcher::fold_events(&events);

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, AssetChangeKind::Added);
    assert_eq!(changes[0].uri, uri);
    assert_eq!(changes[0].previous_uri, None);
}

#[test]
fn owned_and_borrowed_rename_chains_preserve_the_original_uri() {
    let a = AssetUri::parse("res://materials/a.zmaterial").unwrap();
    let b = AssetUri::parse("res://materials/b.zmaterial").unwrap();
    let c = AssetUri::parse("res://materials/c.zmaterial").unwrap();
    let unrelated = AssetUri::parse("res://materials/unrelated.zmaterial").unwrap();
    for modify_between_renames in [false, true] {
        let mut events = vec![
            AssetWatchEvent::Modified(unrelated.clone()),
            AssetWatchEvent::Renamed {
                from: a.clone(),
                to: b.clone(),
            },
        ];
        if modify_between_renames {
            events.push(AssetWatchEvent::Modified(b.clone()));
        }
        events.push(AssetWatchEvent::Renamed {
            from: b.clone(),
            to: c.clone(),
        });
        let borrowed = AssetWatcher::fold_events(&events);
        let mut folded = super::FoldedAssetChangeMap::new();
        for event in events {
            super::fold_event(&mut folded, event);
        }
        let owned = super::finish_folded_events(folded);
        let expected = vec![
            super::AssetChange::new(AssetChangeKind::Renamed, c.clone(), Some(a.clone())),
            super::AssetChange::new(AssetChangeKind::Modified, unrelated.clone(), None),
        ];
        assert_eq!(borrowed, expected);
        assert_eq!(owned, expected);
    }
}

#[test]
fn owned_and_borrowed_rename_folding_preserves_added_and_modified_semantics() {
    let a = AssetUri::parse("res://materials/a.zmaterial").unwrap();
    let b = AssetUri::parse("res://materials/b.zmaterial").unwrap();
    let c = AssetUri::parse("res://materials/c.zmaterial").unwrap();
    let cases = [
        (
            vec![
                AssetWatchEvent::Added(a.clone()),
                AssetWatchEvent::Modified(a.clone()),
                AssetWatchEvent::Modified(a.clone()),
            ],
            super::AssetChange::new(AssetChangeKind::Added, a.clone(), None),
        ),
        (
            vec![
                AssetWatchEvent::Added(a.clone()),
                AssetWatchEvent::Renamed {
                    from: a.clone(),
                    to: b.clone(),
                },
                AssetWatchEvent::Renamed {
                    from: b.clone(),
                    to: c.clone(),
                },
            ],
            super::AssetChange::new(AssetChangeKind::Renamed, c.clone(), Some(a.clone())),
        ),
        (
            vec![
                AssetWatchEvent::Renamed {
                    from: a.clone(),
                    to: b.clone(),
                },
                AssetWatchEvent::Renamed {
                    from: b.clone(),
                    to: c.clone(),
                },
                AssetWatchEvent::Modified(c.clone()),
            ],
            super::AssetChange::new(AssetChangeKind::Modified, c.clone(), Some(a.clone())),
        ),
        (
            vec![
                AssetWatchEvent::Renamed {
                    from: a.clone(),
                    to: b.clone(),
                },
                AssetWatchEvent::Removed(b.clone()),
            ],
            super::AssetChange::new(AssetChangeKind::Removed, b.clone(), None),
        ),
    ];
    for (events, expected) in cases {
        let borrowed = AssetWatcher::fold_events(&events);
        let mut folded = super::FoldedAssetChangeMap::new();
        for event in events {
            super::fold_event(&mut folded, event);
        }
        let owned = super::finish_folded_events(folded);
        assert_eq!(borrowed, vec![expected.clone()]);
        assert_eq!(owned, vec![expected]);
    }
}
