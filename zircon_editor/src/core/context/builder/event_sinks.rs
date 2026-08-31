//! Message-bus adapters for context-owned log, locale, and transaction services.

use crate::core::editing::engine::{
    TransactionEvent, TransactionEventDelivery, TransactionEventKind, TransactionEventSink,
};
use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TransactionMessage,
};
use crate::core::i18n::{EditorI18nEventSink, EditorLocale, LocaleChangeDelivery};
use crate::core::logging::{EditorLogEventSink, LogEventDelivery, LogRecord};

pub(super) struct EditorMessageTransactionEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

pub(super) const LOG_RECORD_EVENT_SCHEMA: &str = "zircon.editor.log.recorded.v1";
pub(super) const LOG_RESYNC_EVENT_SCHEMA: &str = "zircon.editor.log.resync.v1";
pub(super) const I18N_LOCALE_CHANGED_EVENT_SCHEMA: &str = "zircon.editor.i18n.locale-changed.v1";
pub(super) const I18N_LOCALE_RESYNC_EVENT_SCHEMA: &str = "zircon.editor.i18n.locale-resync.v1";

pub(super) struct EditorMessageLogEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

impl EditorMessageLogEventSink {
    pub(super) fn new(bus: SharedEditorMessageBus) -> Self {
        Self {
            bus,
            topic: EditorTopic::log(),
        }
    }
}

impl EditorLogEventSink for EditorMessageLogEventSink {
    fn publish(&self, record: &LogRecord) -> LogEventDelivery {
        let report = self.bus.publish(
            self.topic.clone(),
            EditorMessage::custom(
                LOG_RECORD_EVENT_SCHEMA,
                serde_json::json!({ "sequence": record.sequence() }),
            ),
        );
        if report.error().is_some() || !report.dropped().is_empty() {
            LogEventDelivery::Rejected
        } else if report.backpressured().is_empty() {
            LogEventDelivery::Delivered
        } else {
            LogEventDelivery::Backpressured
        }
    }

    fn resync_required(&self, through_sequence: u64) -> LogEventDelivery {
        let report = self.bus.publish(
            self.topic.clone(),
            EditorMessage::custom(
                LOG_RESYNC_EVENT_SCHEMA,
                serde_json::json!({
                    "through_sequence": through_sequence,
                }),
            ),
        );
        if report.error().is_some() {
            LogEventDelivery::Rejected
        } else if !report.backpressured().is_empty() {
            LogEventDelivery::Backpressured
        } else if !canonical_resync_replaces_every_dropped_delivery(&report) {
            LogEventDelivery::Rejected
        } else if report.delivered().is_empty() {
            LogEventDelivery::NotConfigured
        } else {
            LogEventDelivery::Delivered
        }
    }
}

pub(super) struct EditorMessageI18nEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

impl EditorMessageI18nEventSink {
    pub(super) fn new(bus: SharedEditorMessageBus) -> Self {
        Self {
            bus,
            topic: EditorTopic::i18n(),
        }
    }
}

impl EditorI18nEventSink for EditorMessageI18nEventSink {
    fn locale_changed(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        let report = self.bus.publish(
            self.topic.clone(),
            EditorMessage::custom(
                I18N_LOCALE_CHANGED_EVENT_SCHEMA,
                serde_json::json!({ "locale": locale.as_str() }),
            ),
        );
        if report.error().is_some() || !report.dropped().is_empty() {
            LocaleChangeDelivery::Rejected
        } else if report.backpressured().is_empty() {
            LocaleChangeDelivery::Delivered
        } else {
            LocaleChangeDelivery::Backpressured
        }
    }

    fn locale_resync_required(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        let report = self.bus.publish(
            self.topic.clone(),
            EditorMessage::custom(
                I18N_LOCALE_RESYNC_EVENT_SCHEMA,
                serde_json::json!({ "locale": locale.as_str() }),
            ),
        );
        if report.error().is_some() {
            LocaleChangeDelivery::Rejected
        } else if !report.backpressured().is_empty() {
            LocaleChangeDelivery::Backpressured
        } else if !canonical_resync_replaces_every_dropped_delivery(&report) {
            LocaleChangeDelivery::Rejected
        } else if report.delivered().is_empty() {
            LocaleChangeDelivery::NotConfigured
        } else {
            LocaleChangeDelivery::Delivered
        }
    }
}

fn canonical_resync_replaces_every_dropped_delivery(
    report: &crate::core::editor_message::EditorMessageDispatchReport,
) -> bool {
    // A bounded inbox can evict an old fact and enqueue this marker in one publish. The marker
    // is complete only when every evicted subscriber appears in that same publish's delivery set.
    report
        .dropped()
        .iter()
        .all(|subscriber| report.delivered().contains(subscriber))
}

impl EditorMessageTransactionEventSink {
    pub(super) fn new(bus: SharedEditorMessageBus) -> Self {
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
