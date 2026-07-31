use std::sync::Arc;

use crate::core::commands::{CommandEvalSnapshotHandle, EditorCommandRegistryHandle};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    EditorTransactionEngine, TransactionEvent, TransactionEventDelivery, TransactionEventKind,
    TransactionEventSink,
};
use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TransactionMessage,
};
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::jobs::{EditorJobLimits, EditorJobSystem};
use crate::core::notifications::EditorNotificationService;
use zircon_runtime::core::runtime::tasks::JobScheduler;

use super::{EditorContext, ToolSchedulerService};

struct EditorMessageTransactionEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

impl EditorMessageTransactionEventSink {
    fn new(bus: SharedEditorMessageBus) -> Self {
        Self {
            bus,
            topic: EditorTopic::transaction(),
        }
    }
}

impl TransactionEventSink for EditorMessageTransactionEventSink {
    fn publish(&self, event: TransactionEvent) -> TransactionEventDelivery {
        let message = transaction_message(event);
        let report = self.bus.publish(
            self.topic.clone(),
            EditorMessage::new(EditorMessagePayload::Transaction(message)),
        );
        if report.error().is_some() || !report.dropped().is_empty() {
            TransactionEventDelivery::Rejected
        } else if report.backpressured().is_empty() {
            TransactionEventDelivery::Delivered
        } else {
            TransactionEventDelivery::Backpressured
        }
    }
}

fn transaction_message(event: TransactionEvent) -> TransactionMessage {
    let TransactionEvent {
        transaction,
        history,
        label,
        timestamp_frame,
        kind,
    } = event;
    match kind {
        TransactionEventKind::Started => TransactionMessage::Started {
            transaction,
            history,
            label,
            timestamp_frame,
        },
        TransactionEventKind::Canceled => TransactionMessage::Canceled {
            transaction,
            history,
            label,
            timestamp_frame,
        },
        TransactionEventKind::Committed => TransactionMessage::Committed {
            transaction,
            history,
            label,
            timestamp_frame,
        },
        TransactionEventKind::UndoApplied => TransactionMessage::Undone {
            transaction,
            history,
            label,
            timestamp_frame,
        },
        TransactionEventKind::RedoApplied => TransactionMessage::Redone {
            transaction,
            history,
            label,
            timestamp_frame,
        },
    }
}

pub struct EditorContextBuilder {
    bus: SharedEditorMessageBus,
    scheduler: JobScheduler,
    gateway: EditorRuntimeGatewayHandle,
}

impl EditorContextBuilder {
    pub fn new(scheduler: JobScheduler) -> Self {
        Self {
            bus: SharedEditorMessageBus::default(),
            scheduler,
            gateway: EditorRuntimeGatewayHandle::detached(),
        }
    }

    pub fn with_bus(mut self, bus: SharedEditorMessageBus) -> Self {
        self.bus = bus;
        self
    }

    pub fn with_gateway(mut self, gateway: EditorRuntimeGatewayHandle) -> Self {
        self.gateway = gateway;
        self
    }

    pub fn build(self) -> Arc<EditorContext> {
        let events = Arc::new(EditorEventService::new(self.bus.clone()));
        let jobs = EditorJobSystem::with_scheduler_and_bus(
            self.scheduler,
            self.bus.clone(),
            EditorJobLimits::default(),
        );
        let notifications = EditorNotificationService::default();
        let transactions = EditorTransactionEngine::with_event_sink(
            CoreEditContext::new(self.gateway.clone()),
            Arc::new(EditorMessageTransactionEventSink::new(self.bus.clone())),
        );
        let commands = EditorCommandRegistryHandle::default_workbench();
        let command_eval = CommandEvalSnapshotHandle::default();
        let tools = ToolSchedulerService::new(self.bus.clone());
        Arc::new(EditorContext::new(
            self.bus,
            events,
            jobs,
            notifications,
            transactions,
            commands,
            command_eval,
            tools,
            self.gateway,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editing::engine::{
        HistoryContextId, TransactionEvent, TransactionEventKind, TransactionEventSink,
        TransactionId,
    };
    use crate::core::editor_message::{
        EditorMessageInboxLimits, EditorMessagePayload, EditorTopic, TransactionMessage,
        TOPIC_TRANSACTION,
    };

    use super::EditorMessageTransactionEventSink;

    #[test]
    fn transaction_event_adapter_publishes_every_lifecycle_kind_to_the_canonical_topic() {
        let bus = crate::core::editor_message::SharedEditorMessageBus::default();
        let topic = EditorTopic::parse(TOPIC_TRANSACTION).unwrap();
        let subscriber = bus.register_subscriber([topic.clone()]).unwrap();
        let sink = EditorMessageTransactionEventSink::new(bus.clone());
        let transaction = TransactionId::from_sequence(7);

        for kind in [
            TransactionEventKind::Started,
            TransactionEventKind::Committed,
            TransactionEventKind::UndoApplied,
            TransactionEventKind::RedoApplied,
            TransactionEventKind::Canceled,
        ] {
            assert_eq!(
                sink.publish(TransactionEvent {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                    kind,
                }),
                crate::core::editing::engine::TransactionEventDelivery::Delivered
            );
        }

        let deliveries = bus.drain_deliveries(subscriber);
        assert_eq!(deliveries.len(), 5);
        assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
        assert_eq!(
            deliveries
                .into_iter()
                .map(|delivery| match delivery.message().payload() {
                    EditorMessagePayload::Transaction(message) => message.clone(),
                    payload => panic!("expected transaction payload, received {payload:?}"),
                })
                .collect::<Vec<_>>(),
            vec![
                TransactionMessage::Started {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                },
                TransactionMessage::Committed {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                },
                TransactionMessage::Undone {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                },
                TransactionMessage::Redone {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                },
                TransactionMessage::Canceled {
                    transaction,
                    history: HistoryContextId::Global,
                    label: "Move entity".to_string(),
                    timestamp_frame: 42,
                },
            ]
        );
    }

    #[test]
    fn transaction_event_adapter_reports_canonical_bus_backpressure() {
        let bus = crate::core::editor_message::SharedEditorMessageBus::with_inbox_limits(
            EditorMessageInboxLimits::new(0, 1, 1),
        );
        let topic = EditorTopic::parse(TOPIC_TRANSACTION).unwrap();
        let subscriber = bus.register_subscriber([topic]).unwrap();
        let sink = EditorMessageTransactionEventSink::new(bus.clone());

        assert_eq!(
            sink.publish(TransactionEvent {
                transaction: TransactionId::from_sequence(1),
                history: HistoryContextId::Global,
                label: "Blocked transaction".to_string(),
                timestamp_frame: 0,
                kind: TransactionEventKind::Started,
            }),
            crate::core::editing::engine::TransactionEventDelivery::Backpressured
        );
        assert!(bus.drain_deliveries(subscriber).is_empty());
    }
}
