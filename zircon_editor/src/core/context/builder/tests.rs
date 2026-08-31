use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::event_sinks::{
    EditorMessageI18nEventSink, EditorMessageLogEventSink, EditorMessageTransactionEventSink,
    I18N_LOCALE_CHANGED_EVENT_SCHEMA, I18N_LOCALE_RESYNC_EVENT_SCHEMA, LOG_RECORD_EVENT_SCHEMA,
    LOG_RESYNC_EVENT_SCHEMA,
};
use super::EditorContextBuilder;
use crate::core::editing::engine::{
    HistoryContextId, TransactionEvent, TransactionEventKind, TransactionEventSink, TransactionId,
};
use crate::core::editor_message::{
    EditorMessage, EditorMessageInboxLimits, EditorMessagePayload, EditorTopic,
    SharedEditorMessageBus, TransactionMessage, TOPIC_TRANSACTION,
};
use crate::core::i18n::EditorLocale;
use crate::core::jobs::{EditorJob, EditorJobSpec, JobCategory, JobContext, JobError};
use crate::core::logging::{
    EditorLogEventSink, LogEntry, LogEventDelivery, LogSeverity, LogSource,
};

struct NotificationGateJob {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

impl EditorJob for NotificationGateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        Ok(())
    }
}

#[test]
fn builder_exposes_one_immutable_settings_snapshot_from_its_context() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();

    let first = context.settings().snapshot();
    let second = context.settings().snapshot();

    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(
        context.settings_persistence().diagnostics().queue_entries,
        0
    );
    assert!(context
        .logs()
        .snapshot(&crate::core::logging::LogFilter::default())
        .is_empty());
    assert_eq!(
        context.i18n().translate("command.file.open").as_ref(),
        "Open"
    );
}

#[test]
fn builder_binds_accepted_jobs_to_bounded_progress_notifications() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let ticket = context
        .jobs()
        .submit(
            EditorJobSpec::new("notification progress", JobCategory::Import),
            NotificationGateJob {
                started: started_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    let snapshots = context
        .notifications()
        .progress()
        .snapshot(&context.jobs().progress());
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].job().id(), ticket.id());
    assert_eq!(
        snapshots[0].notification().title_key(),
        "editor.notification.job_progress.title"
    );

    release_sender.send(()).unwrap();
    assert!(ticket.wait().is_ok());
    let deadline = Instant::now() + Duration::from_secs(5);
    while !context.notifications().progress().is_empty() {
        assert!(
            Instant::now() < deadline,
            "finished job binding was not retired"
        );
        std::thread::yield_now();
    }
}

#[test]
fn builder_hot_applies_the_user_locale_setting_to_i18n() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();
    let topic = EditorTopic::i18n();
    let subscriber = context.bus().register_subscriber([topic]).unwrap();
    let locale_key =
        crate::core::settings::SettingsKey::parse(crate::core::settings::EDITOR_LOCALE_KEY)
            .unwrap();

    context
        .settings()
        .set(
            crate::core::settings::SettingsScope::User,
            &locale_key,
            crate::core::settings::SettingValue::Enum("zh-CN".to_owned()),
        )
        .unwrap();

    assert_eq!(context.settings().snapshot().locale(), "zh-CN");
    assert_eq!(context.i18n().active_locale().as_str(), "zh-CN");
    assert!(matches!(
        context.bus().drain_deliveries(subscriber).as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, payload }
                    if schema_id == I18N_LOCALE_CHANGED_EVENT_SCHEMA
                        && payload["locale"] == serde_json::json!("zh-CN"))
    ));

    let subscriber = context
        .bus()
        .register_subscriber([EditorTopic::i18n()])
        .unwrap();
    context
        .settings()
        .clear(crate::core::settings::SettingsScope::User, &locale_key)
        .unwrap();

    assert_eq!(context.settings().snapshot().locale(), "en");
    assert_eq!(context.i18n().active_locale().as_str(), "en");
    assert!(matches!(
        context.bus().drain_deliveries(subscriber).as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, payload }
                    if schema_id == I18N_LOCALE_CHANGED_EVENT_SCHEMA
                        && payload["locale"] == serde_json::json!("en"))
    ));
}

#[test]
fn builder_hot_applies_the_user_autosave_interval_to_the_active_service() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();
    let autosave_key = crate::core::settings::SettingsKey::parse(
        crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
    )
    .unwrap();

    context
        .settings()
        .set(
            crate::core::settings::SettingsScope::User,
            &autosave_key,
            crate::core::settings::SettingValue::Int(60),
        )
        .unwrap();

    assert_eq!(
        context.settings().snapshot().autosave_interval(),
        Duration::from_secs(60)
    );
    assert_eq!(
        context.autosave().policy().interval(),
        Duration::from_secs(60)
    );
}

#[test]
fn builder_initializes_autosave_from_the_persisted_user_setting() {
    let root = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(format!(
            "zircon-editor-autosave-settings-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    let store = crate::core::settings::SettingsStore::from_roots(&root, None);
    let autosave_key = crate::core::settings::SettingsKey::parse(
        crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
    )
    .unwrap();
    let mut settings = crate::core::settings::settings_registry_with_defaults();
    settings
        .set(
            crate::core::settings::SettingsScope::User,
            &autosave_key,
            crate::core::settings::SettingValue::Int(60),
        )
        .unwrap();
    store
        .save_from(crate::core::settings::SettingsScope::User, &settings)
        .unwrap();

    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .with_settings_store(store)
    .build();

    assert_eq!(
        context.autosave().policy().interval(),
        Duration::from_secs(60)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn log_service_publishes_sequence_notifications_to_the_canonical_log_topic() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();
    let topic = EditorTopic::log();
    let subscriber = context.bus().register_subscriber([topic.clone()]).unwrap();
    let report = context
        .logs()
        .emit(
            LogEntry::new(
                LogSource::editor(),
                LogSeverity::Info,
                "context ready",
                7,
                None,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(report.event_delivery(), LogEventDelivery::Delivered);
    let deliveries = context.bus().drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    assert!(matches!(
        deliveries[0].message().payload(),
        EditorMessagePayload::Custom { schema_id, payload }
            if schema_id == LOG_RECORD_EVENT_SCHEMA
                && payload["sequence"] == serde_json::json!(report.record().sequence())
    ));
    assert_eq!(
        context
            .logs()
            .record(report.record().sequence())
            .unwrap()
            .entry()
            .message(),
        "context ready"
    );
}

#[test]
fn i18n_service_publishes_locale_changes_to_the_canonical_topic() {
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .build();
    let topic = EditorTopic::i18n();
    let subscriber = context.bus().register_subscriber([topic.clone()]).unwrap();

    assert!(context
        .i18n()
        .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap());

    let deliveries = context.bus().drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    assert!(matches!(
        deliveries[0].message().payload(),
        EditorMessagePayload::Custom { schema_id, payload }
            if schema_id == I18N_LOCALE_CHANGED_EVENT_SCHEMA
                && payload["locale"] == serde_json::json!("zh-CN")
    ));
}

#[test]
fn bounded_canonical_log_inbox_receives_a_sequence_resync_after_eviction() {
    let bus = SharedEditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(0, 1, 1));
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .with_bus(bus)
    .build();
    let subscriber = context
        .bus()
        .register_subscriber([EditorTopic::log()])
        .unwrap();

    context
        .logs()
        .emit(LogEntry::new(LogSource::editor(), LogSeverity::Info, "first", 1, None).unwrap())
        .unwrap();
    let second = context
        .logs()
        .emit(LogEntry::new(LogSource::editor(), LogSeverity::Info, "second", 2, None).unwrap())
        .unwrap();

    assert_eq!(second.event_delivery(), LogEventDelivery::Rejected);
    let deliveries = context.bus().drain_deliveries(subscriber);
    assert!(matches!(
        deliveries.as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, payload }
                    if schema_id == LOG_RESYNC_EVENT_SCHEMA
                        && payload["through_sequence"] == serde_json::json!(2))
    ));
    let diagnostics = context.logs().diagnostics();
    assert_eq!(diagnostics.resync_required_records, 1);
    assert_eq!(diagnostics.event_resyncs, 1);
    assert_eq!(diagnostics.failed_event_resyncs, 0);
}

#[test]
fn bounded_canonical_i18n_inbox_receives_the_latest_locale_resync_after_eviction() {
    let bus = SharedEditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(0, 1, 1));
    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    )
    .with_bus(bus)
    .build();
    let subscriber = context
        .bus()
        .register_subscriber([EditorTopic::i18n()])
        .unwrap();

    assert!(context
        .i18n()
        .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap());
    assert!(context
        .i18n()
        .set_active_locale(EditorLocale::parse("en").unwrap())
        .unwrap());

    let deliveries = context.bus().drain_deliveries(subscriber);
    assert!(matches!(
        deliveries.as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, payload }
                    if schema_id == I18N_LOCALE_RESYNC_EVENT_SCHEMA
                        && payload["locale"] == serde_json::json!("en"))
    ));
    let diagnostics = context.i18n().event_diagnostics();
    assert_eq!(diagnostics.dropped_events, 1);
    assert_eq!(diagnostics.resyncs, 1);
    assert_eq!(diagnostics.failed_resyncs, 0);
}

#[test]
fn undeliverable_canonical_resync_is_rejected_instead_of_counted_as_unsubscribed() {
    let bus = SharedEditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(0, 0, 1));
    let log_sink = EditorMessageLogEventSink::new(bus.clone());
    let locale_sink = EditorMessageI18nEventSink::new(bus.clone());
    let log_subscriber = bus.register_subscriber([EditorTopic::log()]).unwrap();
    let locale_subscriber = bus.register_subscriber([EditorTopic::i18n()]).unwrap();

    assert_eq!(log_sink.resync_required(1), LogEventDelivery::Rejected);
    assert_eq!(
        crate::core::i18n::EditorI18nEventSink::locale_resync_required(
            &locale_sink,
            &EditorLocale::parse("zh-CN").unwrap(),
        ),
        crate::core::i18n::LocaleChangeDelivery::Rejected
    );
    assert!(bus.drain_deliveries(log_subscriber).is_empty());
    assert!(bus.drain_deliveries(locale_subscriber).is_empty());
}

#[test]
fn canonical_resync_retries_when_only_some_dropped_subscribers_receive_the_marker() {
    let limits = EditorMessageInboxLimits::new(1, 1, 1).with_byte_limits(2_048, 1_200);
    let bus = SharedEditorMessageBus::with_inbox_limits(limits);
    let log_sink = EditorMessageLogEventSink::new(bus.clone());
    let locale_sink = EditorMessageI18nEventSink::new(bus.clone());
    let blocked_log = bus
        .register_subscriber([EditorTopic::transaction(), EditorTopic::log()])
        .unwrap();
    let blocked_locale = bus
        .register_subscriber([EditorTopic::transaction(), EditorTopic::i18n()])
        .unwrap();
    let log_receiver = bus.register_subscriber([EditorTopic::log()]).unwrap();
    let locale_receiver = bus.register_subscriber([EditorTopic::i18n()]).unwrap();
    let lossless = EditorMessage::new(EditorMessagePayload::Transaction(
        TransactionMessage::Started {
            transaction: TransactionId::from_sequence(1),
            history: HistoryContextId::Global,
            label: "x".repeat(1_024),
            timestamp_frame: 1,
        },
    ));

    let report = bus.publish(EditorTopic::transaction(), lossless);
    assert_eq!(report.delivered().len(), 2);
    assert_eq!(log_sink.resync_required(7), LogEventDelivery::Rejected);
    assert_eq!(
        crate::core::i18n::EditorI18nEventSink::locale_resync_required(
            &locale_sink,
            &EditorLocale::parse("zh-CN").unwrap(),
        ),
        crate::core::i18n::LocaleChangeDelivery::Rejected
    );
    assert_eq!(bus.drain_deliveries(blocked_log).len(), 1);
    assert_eq!(bus.drain_deliveries(blocked_locale).len(), 1);
    assert!(matches!(
        bus.drain_deliveries(log_receiver).as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, .. }
                    if schema_id == LOG_RESYNC_EVENT_SCHEMA)
    ));
    assert!(matches!(
        bus.drain_deliveries(locale_receiver).as_slice(),
        [delivery]
            if matches!(delivery.message().payload(),
                EditorMessagePayload::Custom { schema_id, .. }
                    if schema_id == I18N_LOCALE_RESYNC_EVENT_SCHEMA)
    ));
}

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
