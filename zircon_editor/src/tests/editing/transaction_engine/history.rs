use std::sync::atomic::Ordering;

use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistorySaveMarkOutcome,
};
use crate::core::editor_message::DocumentId;

use super::fixture::{DeltaCommand, FixtureContext, finalized_counter};

fn mark_current_saved(engine: &EditorTransactionEngine, history: HistoryContextId) {
    let token = engine.capture_save_token(history).unwrap();
    assert_eq!(
        engine.mark_saved_if_unchanged(history, token).unwrap(),
        HistorySaveMarkOutcome::Marked
    );
}

fn commit_delta(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
    label: &'static str,
    before: u8,
    after: i32,
) {
    let mut scope = engine.begin(label, history).unwrap();
    scope
        .push(DeltaCommand::new(label, before, after, finalized_counter()))
        .unwrap();
    scope.commit().unwrap();
}

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

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 2);
    assert!(!status.can_redo);
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
}

#[test]
fn history_status_is_narrow_and_details_are_explicitly_paged() {
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 256).unwrap();

    for _ in 0..128 {
        commit_delta(
            &engine,
            HistoryContextId::Global,
            "paged history record",
            1,
            1,
        );
    }

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 128);
    assert_eq!(status.generation, 128);
    assert_eq!(status.top.map(|transaction| transaction.raw()), Some(128));
    assert!(status.can_undo);
    assert!(!status.can_redo);
    assert!(status.dirty);

    let first = engine
        .history_details(HistoryContextId::Global, None, 1)
        .unwrap();
    assert_eq!(first.status(), status);
    assert_eq!(first.records().len(), 1);
    assert_eq!(first.records()[0].id.raw(), 1);
    let cursor = first.next_cursor().cloned().unwrap();

    let full_window = engine
        .history_details(HistoryContextId::Global, None, 128)
        .unwrap();
    assert_eq!(full_window.records().len(), 128);
    assert!(full_window.next_cursor().is_none());

    let remaining = engine
        .history_details(HistoryContextId::Global, Some(&cursor), 128)
        .unwrap();
    assert_eq!(remaining.records().len(), 127);
    assert_eq!(remaining.records()[0].id.raw(), 2);

    commit_delta(
        &engine,
        HistoryContextId::Global,
        "invalidate page cursor",
        1,
        1,
    );
    assert!(matches!(
        engine.history_details(HistoryContextId::Global, Some(&cursor), 17),
        Err(EditCommandError::HistoryPageCursorStale {
            history: HistoryContextId::Global,
            cursor_generation: 128,
            current_generation: 129,
        })
    ));
    assert!(matches!(
        engine.history_details(HistoryContextId::Global, None, 0),
        Err(EditCommandError::HistoryPageSizeOutOfRange { requested: 0, .. })
    ));
}

#[test]
fn history_page_cursor_is_bound_to_its_engine_and_history() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let cursor = engine
        .history_details(HistoryContextId::Global, None, 1)
        .unwrap()
        .next_cursor()
        .cloned()
        .unwrap();
    let document = HistoryContextId::Document(DocumentId::new(42));

    assert!(matches!(
        engine.history_details(document, Some(&cursor), 1),
        Err(EditCommandError::HistoryPageCursorHistoryMismatch {
            cursor_history: HistoryContextId::Global,
            requested_history,
        }) if requested_history == document
    ));

    let foreign_engine = EditorTransactionEngine::new(FixtureContext::default());
    assert!(matches!(
        foreign_engine.history_details(HistoryContextId::Global, Some(&cursor), 1),
        Err(EditCommandError::HistoryPageCursorEngineMismatch)
    ));
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

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 1);
    assert_eq!(status.top.map(|transaction| transaction.raw()), Some(2));
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
    mark_current_saved(&engine, HistoryContextId::Global);

    let mut current = engine.begin("current", HistoryContextId::Global).unwrap();
    current
        .push(DeltaCommand::new("current", 2, 2, finalized))
        .unwrap();
    current.commit().unwrap();

    let after_eviction = engine.history_status(HistoryContextId::Global).unwrap();
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

    mark_current_saved(&engine, HistoryContextId::Global);
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
            .history_status(HistoryContextId::Global)
            .unwrap()
            .saved_top_reachable
    );
}

#[test]
fn save_token_rejects_commit_between_capture_and_completion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_rejects_same_top_branch_replacement() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "original", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    engine.undo(HistoryContextId::Global).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "replacement", 0, 2);
    assert_eq!(
        engine
            .history_status(HistoryContextId::Global)
            .unwrap()
            .top
            .map(|transaction| transaction.raw()),
        Some(2)
    );

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn save_token_rejects_undo_and_redo_between_capture_and_completion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);
    let before_undo = engine.capture_save_token(HistoryContextId::Global).unwrap();
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_undo),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));

    let before_redo = engine.capture_save_token(HistoryContextId::Global).unwrap();
    engine.redo(HistoryContextId::Global).unwrap();
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_redo),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn empty_history_token_is_typed_and_invalidated_by_first_commit() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token.clone())
            .unwrap(),
        HistorySaveMarkOutcome::AlreadyMarked
    );

    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn save_token_rejects_cross_document_use() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let first = HistoryContextId::Document(DocumentId::new(1));
    let second = HistoryContextId::Document(DocumentId::new(2));
    let token = engine.capture_save_token(first).unwrap();

    assert!(matches!(
        engine.mark_saved_if_unchanged(second, token),
        Err(EditCommandError::SaveTokenHistoryMismatch {
            token_history,
            requested_history,
        }) if token_history == first && requested_history == second
    ));
}

#[test]
fn invalid_save_token_does_not_flush_an_open_operation_group() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let token_history = HistoryContextId::Document(DocumentId::new(31));
    let token = engine.capture_save_token(token_history).unwrap();
    let group = engine
        .execute_operation(
            "grouped edit",
            HistoryContextId::Global,
            Some("fixture.save-token-side-effect"),
            crate::core::editing::engine::MergeMode::All,
            Box::new(DeltaCommand::new("grouped edit", 1, 1, finalized_counter())),
        )
        .unwrap();

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::SaveTokenHistoryMismatch { .. })
    ));
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(group.transaction_id)
    );
}

#[test]
fn save_token_rejects_cross_engine_use() {
    let first = EditorTransactionEngine::new(FixtureContext::default());
    let second = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&first, HistoryContextId::Global, "same", 0, 1);
    commit_delta(&second, HistoryContextId::Global, "same", 0, 1);
    let foreign = first.capture_save_token(HistoryContextId::Global).unwrap();

    assert!(matches!(
        second.mark_saved_if_unchanged(HistoryContextId::Global, foreign),
        Err(EditCommandError::SaveTokenEngineMismatch)
    ));
    assert!(second.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_capture_and_completion_reject_active_transaction_scopes() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "saved candidate", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    let mut active = engine.begin("active", HistoryContextId::Global).unwrap();
    active
        .push(DeltaCommand::new("active", 1, 2, finalized_counter()))
        .unwrap();
    assert!(matches!(
        engine.capture_save_token(HistoryContextId::Global),
        Err(EditCommandError::SaveTokenActiveTransaction {
            operation: "capture save token",
            ..
        })
    ));
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token.clone()),
        Err(EditCommandError::SaveTokenActiveTransaction {
            operation: "mark saved if unchanged",
            ..
        })
    ));

    active.cancel().unwrap();
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token)
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
}

#[test]
fn repeated_save_completion_is_reported_without_moving_the_baseline() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token.clone())
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token)
            .unwrap(),
        HistorySaveMarkOutcome::AlreadyMarked
    );
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_is_invalidated_by_capacity_eviction_and_history_clear() {
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let before_eviction = engine.capture_save_token(HistoryContextId::Global).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_eviction),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));

    let before_clear = engine.capture_save_token(HistoryContextId::Global).unwrap();
    let mut transition = engine
        .begin_exclusive_transition("clear history for save-token test")
        .unwrap();
    transition
        .clear_history_and_context::<FixtureContext>(
            HistoryContextId::Global,
            "FixtureContext",
            |_| Ok(()),
        )
        .unwrap();
    drop(transition);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_clear),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn multi_document_tokens_complete_independently() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let first = HistoryContextId::Document(DocumentId::new(11));
    let second = HistoryContextId::Document(DocumentId::new(12));
    commit_delta(&engine, first, "first", 0, 1);
    commit_delta(&engine, second, "second", 1, 2);
    let first_token = engine.capture_save_token(first).unwrap();
    let second_token = engine.capture_save_token(second).unwrap();

    commit_delta(&engine, second, "second changed", 2, 3);

    assert_eq!(
        engine.mark_saved_if_unchanged(first, first_token).unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert!(matches!(
        engine.mark_saved_if_unchanged(second, second_token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
    assert!(!engine.is_dirty(first).unwrap());
    assert!(engine.is_dirty(second).unwrap());
}
