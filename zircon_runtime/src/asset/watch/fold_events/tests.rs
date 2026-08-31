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
