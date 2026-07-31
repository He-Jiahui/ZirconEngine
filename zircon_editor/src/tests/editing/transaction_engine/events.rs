use std::sync::{Arc, Mutex};

use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, TransactionEvent, TransactionEventDelivery,
    TransactionEventKind, TransactionEventSink,
};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn transaction_events_are_emitted_directly_to_the_configured_sink_in_lifecycle_order() {
    let finalized = finalized_counter();
    let sink = Arc::new(RecordingEventSink::default());
    let engine = EditorTransactionEngine::with_event_sink(FixtureContext::default(), sink.clone());
    let mut scope = engine.begin("edit", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("edit", 1, 1, finalized.clone()))
        .unwrap();
    scope.commit().unwrap();
    engine.undo(HistoryContextId::Global).unwrap();
    engine.redo(HistoryContextId::Global).unwrap();

    let canceled = engine.begin("canceled", HistoryContextId::Global).unwrap();
    canceled.cancel().unwrap();

    let kinds = sink
        .events()
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

#[derive(Default)]
struct RecordingEventSink {
    events: Mutex<Vec<TransactionEvent>>,
}

impl RecordingEventSink {
    fn events(&self) -> Vec<TransactionEvent> {
        self.events
            .lock()
            .expect("the recording event sink lock should not be poisoned")
            .clone()
    }
}

impl TransactionEventSink for RecordingEventSink {
    fn publish(&self, event: TransactionEvent) -> TransactionEventDelivery {
        self.events
            .lock()
            .expect("the recording event sink lock should not be poisoned")
            .push(event);
        TransactionEventDelivery::Delivered
    }
}
