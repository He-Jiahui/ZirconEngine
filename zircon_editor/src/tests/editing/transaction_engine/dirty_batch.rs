use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistoryDirtyBatchKind,
    HistorySaveMarkOutcome,
};
use crate::core::editor_message::DocumentId;

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

fn document(value: u64) -> HistoryContextId {
    HistoryContextId::Document(DocumentId::new(value))
}

fn commit(engine: &EditorTransactionEngine, history: HistoryContextId, label: &'static str) {
    let mut scope = engine.begin(label, history).unwrap();
    scope
        .push(DeltaCommand::new(label, 0, 1, finalized_counter()))
        .unwrap();
    scope.commit().unwrap();
}

#[test]
fn initial_batch_is_sorted_reset_and_stable_cursor_is_allocation_empty() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let second = document(2);
    commit(&engine, second, "second");

    let initial = engine.dirty_states_since(None).unwrap();
    assert_eq!(initial.kind(), HistoryDirtyBatchKind::Reset);
    assert_eq!(
        initial
            .states()
            .iter()
            .map(|state| state.history())
            .collect::<Vec<_>>(),
        vec![second]
    );
    assert!(initial.states()[0].is_dirty());

    let stable = engine.dirty_states_since(Some(initial.cursor())).unwrap();
    assert_eq!(stable.kind(), HistoryDirtyBatchKind::Unchanged);
    assert!(stable.states().is_empty());
    assert_eq!(engine.take_dirty_journal_visits_for_test(), 0);
}

#[test]
fn delta_contains_only_histories_changed_after_the_cursor() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let second = document(12);
    let baseline = engine.dirty_states_since(None).unwrap();

    commit(&engine, second, "second");

    let delta = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
    assert_eq!(delta.kind(), HistoryDirtyBatchKind::Delta);
    assert_eq!(delta.states().len(), 1);
    assert_eq!(delta.states()[0].history(), second);
    assert!(delta.states()[0].is_dirty());
    assert_eq!(engine.take_dirty_journal_visits_for_test(), 1);
}

#[test]
fn saved_top_change_publishes_clean_delta_without_breaking_idempotent_completion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let history = document(21);
    commit(&engine, history, "dirty");
    let baseline = engine.dirty_states_since(None).unwrap();
    let token = engine.capture_save_token(history).unwrap();

    assert_eq!(
        engine
            .mark_saved_if_unchanged(history, token.clone())
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert_eq!(
        engine.mark_saved_if_unchanged(history, token).unwrap(),
        HistorySaveMarkOutcome::AlreadyMarked
    );

    let delta = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
    assert_eq!(delta.kind(), HistoryDirtyBatchKind::Delta);
    assert_eq!(delta.states().len(), 1);
    assert!(!delta.states()[0].is_dirty());
}

#[test]
fn cursor_from_another_engine_is_rejected() {
    let first = EditorTransactionEngine::new(FixtureContext::default());
    let second = EditorTransactionEngine::new(FixtureContext::default());
    let foreign = first.dirty_states_since(None).unwrap();

    assert!(matches!(
        second.dirty_states_since(Some(foreign.cursor())),
        Err(EditCommandError::HistoryDirtyCursorEngineMismatch)
    ));
}

#[test]
fn cursor_older_than_the_bounded_journal_receives_reset() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let history = document(41);
    let baseline = engine.dirty_states_since(None).unwrap();
    for _ in 0..=4_096 {
        commit(&engine, history, "advance dirty journal");
    }

    let reset = engine.dirty_states_since(Some(baseline.cursor())).unwrap();

    assert_eq!(reset.kind(), HistoryDirtyBatchKind::Reset);
    assert_eq!(reset.states().len(), 1);
    assert_eq!(reset.states()[0].history(), history);
}

#[test]
fn undo_redo_and_history_clear_publish_current_dirty_state() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let history = document(51);
    commit(&engine, history, "saved edit");
    let token = engine.capture_save_token(history).unwrap();
    engine.mark_saved_if_unchanged(history, token).unwrap();
    let saved = engine.dirty_states_since(None).unwrap();

    engine.undo(history).unwrap();
    let undone = engine.dirty_states_since(Some(saved.cursor())).unwrap();
    assert_eq!(undone.kind(), HistoryDirtyBatchKind::Delta);
    assert!(undone.states()[0].is_dirty());

    engine.redo(history).unwrap();
    let redone = engine.dirty_states_since(Some(undone.cursor())).unwrap();
    assert_eq!(redone.kind(), HistoryDirtyBatchKind::Delta);
    assert!(!redone.states()[0].is_dirty());

    let mut transition = engine
        .begin_exclusive_transition("clear history dirty projection")
        .unwrap();
    assert!(transition
        .clear_history_and_context::<FixtureContext>(history, "FixtureContext", |_| Ok(()))
        .unwrap());
    drop(transition);
    let cleared = engine.dirty_states_since(Some(redone.cursor())).unwrap();
    assert_eq!(cleared.kind(), HistoryDirtyBatchKind::Delta);
    assert!(!cleared.states()[0].is_dirty());
}

#[test]
fn failed_generation_reservation_and_clear_type_mismatch_publish_no_delta() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let history = document(61);
    engine.set_dirty_generation_for_test(u64::MAX);
    let baseline = engine.dirty_states_since(None).unwrap();
    let mut scope = engine.begin("exhausted dirty generation", history).unwrap();
    scope
        .push(DeltaCommand::new(
            "exhausted dirty generation",
            0,
            1,
            finalized_counter(),
        ))
        .unwrap();
    assert!(matches!(
        scope.commit(),
        Err(EditCommandError::HistoryDirtyGenerationExhausted)
    ));
    let after_failure = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
    assert_eq!(after_failure.kind(), HistoryDirtyBatchKind::Unchanged);

    let mut transition = engine
        .begin_exclusive_transition("mismatched clear")
        .unwrap();
    assert!(matches!(
        transition.clear_history_and_context::<u32>(history, "u32", |_| Ok(())),
        Err(EditCommandError::ContextTypeMismatch { .. })
    ));
    drop(transition);
    let after_mismatch = engine
        .dirty_states_since(Some(after_failure.cursor()))
        .unwrap();
    assert_eq!(after_mismatch.kind(), HistoryDirtyBatchKind::Unchanged);
}
