use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, TransactionEventKind,
};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn internal_events_follow_started_committed_undo_redo_and_canceled_order() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine.begin("edit", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("edit", 1, 1, finalized.clone()))
        .unwrap();
    scope.commit().unwrap();
    engine.undo(HistoryContextId::Global).unwrap();
    engine.redo(HistoryContextId::Global).unwrap();

    let canceled = engine.begin("canceled", HistoryContextId::Global).unwrap();
    canceled.cancel().unwrap();

    let kinds = engine
        .drain_events()
        .unwrap()
        .into_iter()
        .map(|event| event.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            TransactionEventKind::Started,
            TransactionEventKind::Committed,
            TransactionEventKind::UndoApplied,
            TransactionEventKind::RedoApplied,
            TransactionEventKind::Started,
            TransactionEventKind::Canceled,
        ]
    );
}
