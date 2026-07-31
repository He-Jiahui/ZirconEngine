use std::sync::atomic::Ordering;

use crate::core::editing::engine::{EditCommandError, EditorTransactionEngine, HistoryContextId};

use super::fixture::{DeltaCommand, FixtureContext, finalized_counter};

fn engine_with_committed_history() -> (
    EditorTransactionEngine,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("exclusive transition seed", HistoryContextId::Global)
        .expect("the seed transaction should begin");
    scope
        .push(DeltaCommand::new("exclusive transition seed", 1, 5, finalized.clone()).selecting(7))
        .expect("the seed transaction should accept its command");
    scope
        .commit()
        .expect("the seed transaction should commit into global history");
    (engine, finalized)
}

#[test]
fn failed_exclusive_context_update_preserves_context_selection_and_history() {
    let (engine, finalized) = engine_with_committed_history();
    let before_history = engine
        .history_status(HistoryContextId::Global)
        .expect("the seeded history should be readable");

    let error = {
        let mut transition = engine
            .begin_exclusive_transition("failed context update")
            .expect("the transition should reserve the engine");
        transition
            .clear_history_and_context::<FixtureContext>(
                HistoryContextId::Global,
                "FixtureContext",
                |context| {
                    context.selection = 99;
                    Err(EditCommandError::InvariantViolation {
                        invariant: "fixture context update failure",
                    })
                },
            )
            .expect_err("a failed context update should reach the caller")
    };

    assert!(matches!(
        error,
        EditCommandError::InvariantViolation {
            invariant: "fixture context update failure"
        }
    ));
    assert_eq!(
        engine
            .history_status(HistoryContextId::Global)
            .expect("a failed update must retain history"),
        before_history
    );
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .expect("a failed update must restore the context"),
        Some((5, 7))
    );
    assert_eq!(finalized.load(Ordering::SeqCst), 0);
    assert!(
        engine
            .undo(HistoryContextId::Global)
            .expect("the preserved history should remain undoable")
    );
}

#[test]
fn successful_exclusive_context_update_clears_history_after_update() {
    let (engine, finalized) = engine_with_committed_history();

    let changed = {
        let mut transition = engine
            .begin_exclusive_transition("successful context update")
            .expect("the transition should reserve the engine");
        transition
            .clear_history_and_context::<FixtureContext>(
                HistoryContextId::Global,
                "FixtureContext",
                |context| {
                    context.value = 11;
                    context.selection = 13;
                    Ok(())
                },
            )
            .expect("a successful context update should clear the old history")
    };

    assert!(changed);
    let history = engine
        .history_status(HistoryContextId::Global)
        .expect("the cleared history should be readable");
    assert_eq!(history.len, 0);
    assert!(!history.can_undo);
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .expect("the successful update should retain its context mutation"),
        Some((11, 13))
    );
}

#[test]
fn failed_exclusive_context_update_faults_the_engine_when_selection_rollback_fails() {
    let (engine, _) = engine_with_committed_history();
    engine
        .with_context_mut::<FixtureContext, _>(|context| {
            context.fail_selection_restore = Some(7);
        })
        .expect("the fixture context should accept rollback failure setup");

    let error = {
        let mut transition = engine
            .begin_exclusive_transition("failed context update with failed rollback")
            .expect("the transition should reserve the engine");
        transition
            .clear_history_and_context::<FixtureContext>(
                HistoryContextId::Global,
                "FixtureContext",
                |context| {
                    context.selection = 99;
                    Err(EditCommandError::InvariantViolation {
                        invariant: "fixture context update failure",
                    })
                },
            )
            .expect_err("a failed rollback must fault the transaction engine")
    };

    assert!(matches!(error, EditCommandError::RollbackFailed { .. }));
    assert!(matches!(
        engine.begin("after failed transition rollback", HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}
