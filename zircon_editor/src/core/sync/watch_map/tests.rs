use zircon_runtime_interface::world_sync::{InvalidationBatch, WatchToken};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;

use super::{WorldWatchMap, WorldWatchMapError};

fn view(name: &str) -> ViewInstanceId {
    ViewInstanceId::new(name)
}

#[test]
fn binding_a_token_replaces_both_sides_of_the_old_relation() {
    let mut map = WorldWatchMap::default();
    let token = WatchToken::new(7);
    let hierarchy = view("hierarchy");
    let inspector = view("inspector");

    map.bind(
        token,
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    let replaced = map
        .bind(
            token,
            inspector.clone(),
            EditorViewInvalidationMask::PRESENTATION_DATA,
        )
        .unwrap()
        .unwrap();

    assert_eq!(replaced.view(), &hierarchy);
    assert_eq!(map.tokens_for_view(&hierarchy).count(), 0);
    assert_eq!(
        map.tokens_for_view(&inspector).collect::<Vec<_>>(),
        vec![token]
    );
    assert_eq!(map.binding(token).unwrap().view(), &inspector);
}

#[test]
fn unbinding_a_view_returns_sorted_runtime_tokens_and_clears_reverse_state() {
    let mut map = WorldWatchMap::default();
    let hierarchy = view("hierarchy");
    let inspector = view("inspector");
    for token in [WatchToken::new(9), WatchToken::new(3), WatchToken::new(5)] {
        map.bind(
            token,
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    }
    map.bind(
        WatchToken::new(11),
        inspector,
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    assert_eq!(
        map.unbind_view(&hierarchy),
        vec![WatchToken::new(3), WatchToken::new(5), WatchToken::new(9)]
    );
    assert_eq!(map.len(), 1);
    assert!(map.binding(WatchToken::new(3)).is_none());
    assert_eq!(map.tokens_for_view(&hierarchy).count(), 0);
}

#[test]
fn project_coalesces_masks_per_view_and_reports_duplicate_and_unknown_tokens() {
    let mut map = WorldWatchMap::default();
    let hierarchy = view("hierarchy");
    map.bind(
        WatchToken::new(1),
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    map.bind(
        WatchToken::new(2),
        hierarchy.clone(),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    let projection = map.project(&InvalidationBatch {
        generation: 42,
        dirty: vec![
            WatchToken::new(2),
            WatchToken::new(99),
            WatchToken::new(1),
            WatchToken::new(2),
            WatchToken::new(99),
        ],
        facts: Vec::new(),
    });

    assert_eq!(projection.generation(), 42);
    assert_eq!(projection.matched_tokens(), 2);
    assert_eq!(
        projection.duplicate_tokens(),
        &[WatchToken::new(2), WatchToken::new(99)]
    );
    assert_eq!(projection.unknown_tokens(), &[WatchToken::new(99)]);
    assert_eq!(projection.dirty().len(), 1);
    assert_eq!(
        projection.dirty().mask_for(&hierarchy),
        Some(
            EditorViewInvalidationMask::TREE_STRUCTURE
                .union(EditorViewInvalidationMask::PRESENTATION_DATA)
        )
    );
}

#[test]
fn projection_borrows_bound_view_ids_while_coalescing_masks() {
    let source = include_str!("../watch_map.rs");

    assert!(source.contains("dirty.mark_ref(binding.view(), binding.mask)"));
    assert!(!source.contains("dirty.mark(binding.view.clone(), binding.mask)"));
}

#[test]
fn draining_tokens_clears_the_session_owned_map() {
    let mut map = WorldWatchMap::default();
    map.bind(
        WatchToken::new(4),
        view("hierarchy"),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    map.bind(
        WatchToken::new(2),
        view("inspector"),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    assert_eq!(
        map.drain_tokens(),
        vec![WatchToken::new(2), WatchToken::new(4)]
    );
    assert!(map.is_empty());
    assert!(map.drain_tokens().is_empty());
}

#[test]
fn empty_masks_are_rejected_without_mutating_indexes() {
    let mut map = WorldWatchMap::default();

    assert_eq!(
        map.bind(
            WatchToken::new(1),
            view("hierarchy"),
            EditorViewInvalidationMask::NONE,
        ),
        Err(WorldWatchMapError::EmptyInvalidationMask)
    );
    assert_eq!(
        map.bind(
            WatchToken::new(0),
            view("hierarchy"),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        ),
        Err(WorldWatchMapError::InvalidToken)
    );
    assert!(map.is_empty());
}

#[test]
fn invalid_rebind_preserves_the_existing_relation() {
    let mut map = WorldWatchMap::default();
    let token = WatchToken::new(1);
    let hierarchy = view("hierarchy");
    map.bind(
        token,
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();

    assert_eq!(
        map.bind(token, view("inspector"), EditorViewInvalidationMask::NONE,),
        Err(WorldWatchMapError::EmptyInvalidationMask)
    );
    assert_eq!(map.binding(token).unwrap().view(), &hierarchy);
    assert_eq!(
        map.tokens_for_view(&hierarchy).collect::<Vec<_>>(),
        vec![token]
    );
}

#[test]
fn unbind_token_cleans_reverse_state_and_unknown_token_is_a_no_op() {
    let mut map = WorldWatchMap::default();
    let token = WatchToken::new(7);
    let hierarchy = view("hierarchy");
    map.bind(
        token,
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();

    let removed = map.unbind_token(token).unwrap();
    assert_eq!(removed.token(), token);
    assert_eq!(removed.view(), &hierarchy);
    assert!(map.binding(token).is_none());
    assert_eq!(map.tokens_for_view(&hierarchy).count(), 0);
    assert!(map.unbind_token(token).is_none());
    assert!(map.is_empty());
}
