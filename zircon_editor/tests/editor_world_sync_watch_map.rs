use zircon_editor::core::editor_event::ViewInstanceId;
use zircon_editor::core::editor_message::EditorViewInvalidationMask;
use zircon_editor::core::sync::WorldWatchMap;
use zircon_runtime_interface::world_sync::{InvalidationBatch, WatchToken};

#[test]
fn public_watch_map_projects_runtime_tokens_into_view_dirty_state() {
    let mut map = WorldWatchMap::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    map.bind(
        WatchToken::new(4),
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    map.bind(
        WatchToken::new(8),
        hierarchy.clone(),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    let projection = map.project(&InvalidationBatch {
        generation: 17,
        dirty: vec![WatchToken::new(8), WatchToken::new(99), WatchToken::new(4)],
        facts: Vec::new(),
    });

    assert_eq!(projection.generation(), 17);
    assert_eq!(projection.matched_tokens(), 2);
    assert_eq!(projection.unknown_tokens(), &[WatchToken::new(99)]);
    assert_eq!(
        projection.dirty().mask_for(&hierarchy),
        Some(
            EditorViewInvalidationMask::TREE_STRUCTURE
                .union(EditorViewInvalidationMask::PRESENTATION_DATA)
        )
    );
}

#[test]
fn public_watch_map_view_and_session_cleanup_return_sorted_tokens() {
    let mut map = WorldWatchMap::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let inspector = ViewInstanceId::new("inspector");
    for token in [WatchToken::new(8), WatchToken::new(3)] {
        map.bind(
            token,
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    }
    map.bind(
        WatchToken::new(5),
        inspector,
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    assert_eq!(
        map.unbind_view(&hierarchy),
        vec![WatchToken::new(3), WatchToken::new(8)]
    );
    assert_eq!(map.drain_tokens(), vec![WatchToken::new(5)]);
    assert!(map.is_empty());
}
