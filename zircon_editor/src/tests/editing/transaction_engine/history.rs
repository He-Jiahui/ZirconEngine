use std::sync::atomic::Ordering;

use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn push_after_undo_truncates_redo_and_finalizes_it() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    for label in ["one", "two"] {
        let mut scope = engine.begin(label, HistoryContextId::Global).unwrap();
        scope
            .push(DeltaCommand::new(label, 1, 1, finalized.clone()))
            .unwrap();
        scope.commit().unwrap();
    }
    engine.undo(HistoryContextId::Global).unwrap();

    let mut replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement
        .push(DeltaCommand::new("replacement", 2, 5, finalized.clone()))
        .unwrap();
    replacement.commit().unwrap();

    let snapshot = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(snapshot.len, 2);
    assert!(!snapshot.can_redo);
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
}

#[test]
fn capacity_evicts_the_oldest_record_and_finalizes_it() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();

    for label in ["one", "two"] {
        let mut scope = engine.begin(label, HistoryContextId::Global).unwrap();
        scope
            .push(DeltaCommand::new(label, 1, 1, finalized.clone()))
            .unwrap();
        scope.commit().unwrap();
    }

    let snapshot = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(snapshot.len, 1);
    assert_eq!(snapshot.top, Some(0));
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
}

#[test]
fn eviction_of_saved_first_record_keeps_the_saved_baseline_reachable() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();

    let mut saved = engine.begin("saved", HistoryContextId::Global).unwrap();
    saved
        .push(DeltaCommand::new("saved", 1, 1, finalized.clone()))
        .unwrap();
    saved.commit().unwrap();
    engine.mark_saved(HistoryContextId::Global).unwrap();

    let mut current = engine.begin("current", HistoryContextId::Global).unwrap();
    current
        .push(DeltaCommand::new("current", 2, 2, finalized))
        .unwrap();
    current.commit().unwrap();

    let after_eviction = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(after_eviction.saved_top, None);
    assert!(after_eviction.saved_top_reachable);
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    engine.undo(HistoryContextId::Global).unwrap();
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(1)
    );
}

#[test]
fn saved_top_is_the_dirty_state_authority_even_after_branching() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());

    let mut first = engine.begin("one", HistoryContextId::Global).unwrap();
    first
        .push(DeltaCommand::new("one", 1, 1, finalized.clone()))
        .unwrap();
    first.commit().unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    engine.mark_saved(HistoryContextId::Global).unwrap();
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    let mut branch = engine.begin("branch", HistoryContextId::Global).unwrap();
    branch
        .push(DeltaCommand::new("branch", 2, 3, finalized))
        .unwrap();
    branch.commit().unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());
    assert!(
        !engine
            .history_snapshot(HistoryContextId::Global)
            .unwrap()
            .saved_top_reachable
    );
}
