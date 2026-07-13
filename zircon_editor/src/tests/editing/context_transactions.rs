use std::ptr;

use crate::core::context::EditorContextBuilder;
use crate::core::editing::engine::{HistoryContextId, TransactionEventKind};
use crate::core::jobs::test_job_scheduler;

#[test]
fn editor_context_exposes_one_transaction_engine_instance() {
    let context = EditorContextBuilder::new(test_job_scheduler()).build();

    assert!(ptr::eq(context.transactions(), context.transactions()));
}

#[test]
fn committing_an_empty_context_transaction_does_not_create_history() {
    let context = EditorContextBuilder::new(test_job_scheduler()).build();
    let transaction = context
        .transactions()
        .begin("Metadata-only dispatch", HistoryContextId::Global)
        .expect("begin empty transaction");

    transaction.commit().expect("commit empty transaction");

    let history = context
        .transactions()
        .history_snapshot(HistoryContextId::Global)
        .expect("query global history");
    assert_eq!(history.len, 0);
    assert!(!history.can_undo);
    assert!(!history.can_redo);
    let events = context
        .transactions()
        .drain_events()
        .expect("drain transaction events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, TransactionEventKind::Started);
    assert_eq!(events[1].kind, TransactionEventKind::Committed);
}
