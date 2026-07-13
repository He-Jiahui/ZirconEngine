use std::sync::atomic::Ordering;

use crate::core::editing::engine::{EditCommandError, EditorTransactionEngine, HistoryContextId};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn undo_selection_restore_failure_recovers_context_and_keeps_cursor() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext {
        fail_selection_restore: Some(0),
        ..FixtureContext::default()
    });
    let mut scope = engine.begin("selection", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("selection", 1, 1, finalized).selecting(9))
        .unwrap();
    scope.commit().unwrap();

    assert!(matches!(
        engine.undo(HistoryContextId::Global),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    let history = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(history.top, Some(0));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .unwrap(),
        Some((1, 9))
    );
}

#[test]
fn redo_selection_restore_failure_recovers_context_and_keeps_cursor() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext {
        fail_selection_restore: Some(9),
        ..FixtureContext::default()
    });
    let mut scope = engine.begin("selection", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("selection", 1, 1, finalized).selecting(9))
        .unwrap();
    scope.commit().unwrap();
    engine.undo(HistoryContextId::Global).unwrap();

    assert!(matches!(
        engine.redo(HistoryContextId::Global),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    let history = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(history.top, None);
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .unwrap(),
        Some((0, 0))
    );
}

#[test]
fn apply_error_after_mutation_reverts_the_failing_command() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("apply failure", HistoryContextId::Global)
        .unwrap();
    assert!(matches!(
        scope.push(DeltaCommand::new("mutating failure", 1, 5, finalized).mutating_then_failing()),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );
    let replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement.cancel().unwrap();
}

#[test]
fn apply_error_before_mutation_does_not_run_reverse_effect() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("unchanged apply failure", HistoryContextId::Global)
        .unwrap();
    assert!(matches!(
        scope.push(DeltaCommand::new("unchanged", 1, 5, finalized).failing()),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );
}

#[test]
fn revert_error_before_mutation_retains_applied_state_and_faults_engine() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("unchanged revert failure", HistoryContextId::Global)
        .unwrap();
    scope
        .push(
            DeltaCommand::new("unchanged revert failure", 1, 5, finalized.clone())
                .revert_failing_before_mutation(),
        )
        .unwrap();
    assert!(matches!(
        scope.cancel(),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(finalized.load(Ordering::SeqCst), 0);
    assert!(matches!(
        engine.history_snapshot(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}

#[test]
fn revert_error_retains_commands_and_faults_engine_without_finalizing() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("revert failure", HistoryContextId::Global)
        .unwrap();
    scope
        .push(
            DeltaCommand::new("revert failure", 1, 5, finalized.clone())
                .revert_failing_after_mutation(),
        )
        .unwrap();
    assert!(matches!(
        scope.cancel(),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(finalized.load(Ordering::SeqCst), 0);
    assert!(matches!(
        engine.history_snapshot(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}
