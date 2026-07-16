use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, MergeMode, TransactionEventKind,
};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn operation_group_keeps_one_transaction_and_uses_command_merge() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let finalized = finalized_counter();

    let first = engine
        .execute_operation(
            "Drag Value",
            HistoryContextId::Global,
            Some("fixture.drag.7"),
            MergeMode::All,
            Box::new(DeltaCommand::new("delta", 7, 2, finalized.clone())),
        )
        .unwrap();
    let second = engine
        .execute_operation(
            "Drag Value",
            HistoryContextId::Global,
            Some("fixture.drag.7"),
            MergeMode::All,
            Box::new(DeltaCommand::new("delta", 7, 3, finalized.clone())),
        )
        .unwrap();

    assert_eq!(first.transaction_id, second.transaction_id);
    assert!(first.group_open && second.group_open);
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(first.transaction_id)
    );
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(5)
    );
    let history = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(history.len, 1);
    assert_eq!(history.records[0].command_count, 1);
    assert_eq!(finalized.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(
        engine
            .drain_events()
            .unwrap()
            .into_iter()
            .map(|event| event.kind)
            .collect::<Vec<_>>(),
        vec![
            TransactionEventKind::Started,
            TransactionEventKind::Committed
        ]
    );
}

#[test]
fn operation_group_switch_commits_previous_and_undo_flushes_current() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let finalized = finalized_counter();

    let first = engine
        .execute_operation(
            "First Group",
            HistoryContextId::Global,
            Some("fixture.first"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new("first", 1, 4, finalized.clone())),
        )
        .unwrap();
    let second = engine
        .execute_operation(
            "Second Group",
            HistoryContextId::Global,
            Some("fixture.second"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new("second", 2, 6, finalized)),
        )
        .unwrap();

    assert_ne!(first.transaction_id, second.transaction_id);
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(4)
    );
    let history = engine.history_snapshot(HistoryContextId::Global).unwrap();
    assert_eq!(history.len, 2);
    assert!(history.can_redo);
}
