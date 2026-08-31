use std::ptr;

use crate::core::context::EditorContextBuilder;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TransactionMessage,
    TOPIC_TRANSACTION,
};
use crate::core::jobs::test_job_scheduler;

#[test]
fn editor_context_exposes_one_transaction_engine_instance() {
    let context = EditorContextBuilder::new(test_job_scheduler(), test_job_scheduler()).build();

    assert!(ptr::eq(context.transactions(), context.transactions()));
}

#[test]
fn committing_an_empty_context_transaction_does_not_create_history() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_TRANSACTION).unwrap();
    let subscriber = bus.register_subscriber([topic.clone()]).unwrap();
    let context = EditorContextBuilder::new(test_job_scheduler(), test_job_scheduler())
        .with_bus(bus.clone())
        .build();
    let transaction = context
        .transactions()
        .begin("Metadata-only dispatch", HistoryContextId::Global)
        .expect("begin empty transaction");

    transaction.commit().expect("commit empty transaction");

    let history = context
        .transactions()
        .history_status(HistoryContextId::Global)
        .expect("query global history");
    assert_eq!(history.len, 0);
    assert!(!history.can_undo);
    assert!(!history.can_redo);
    let deliveries = bus.drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 2);
    assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
    assert!(matches!(
        deliveries[0].message().payload(),
        EditorMessagePayload::Transaction(TransactionMessage::Started { .. })
    ));
    assert!(matches!(
        deliveries[1].message().payload(),
        EditorMessagePayload::Transaction(TransactionMessage::Committed { .. })
    ));
}
