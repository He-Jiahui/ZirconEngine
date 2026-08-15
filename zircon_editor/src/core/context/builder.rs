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
use crate::core::i18n::{
    EditorI18nEventSink, EditorI18nService, EditorLocale, LocaleChangeDelivery,
};
use crate::core::jobs::{
    register_editor_job_quota_settings, resolve_editor_job_limits, EditorJobSystem,
};
use crate::core::logging::{EditorLogEventSink, EditorLogService, LogEventDelivery, LogRecord};
use crate::core::notifications::EditorNotificationService;
use crate::core::recovery::{AutosavePolicy, EditorAutosaveService};
use crate::core::settings::{
    settings_registry_with_defaults, SettingChange, SettingsChangeSubscriber,
    SettingsPersistenceService, SettingsSnapshot, SettingsStartup, SettingsStore,
    SettingsUserLayerLoad, EDITOR_LOCALE_KEY,
};
use zircon_runtime::core::runtime::tasks::JobScheduler;

use super::{EditorContext, ToolSchedulerService};

struct EditorMessageTransactionEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

const LOG_RECORD_EVENT_SCHEMA: &str = "zircon.editor.log.recorded.v1";
const LOG_RESYNC_EVENT_SCHEMA: &str = "zircon.editor.log.resync.v1";
const I18N_LOCALE_CHANGED_EVENT_SCHEMA: &str = "zircon.editor.i18n.locale-changed.v1";
const I18N_LOCALE_RESYNC_EVENT_SCHEMA: &str = "zircon.editor.i18n.locale-resync.v1";

struct EditorMessageLogEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

impl EditorMessageLogEventSink {
    fn new(bus: SharedEditorMessageBus) -> Self {
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

struct EditorMessageI18nEventSink {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

struct EditorI18nSettingsChangeSubscriber {
    i18n: Arc<EditorI18nService>,
}

impl SettingsChangeSubscriber for EditorI18nSettingsChangeSubscriber {
    fn settings_changed(&self, changes: &[SettingChange], snapshot: &SettingsSnapshot) {
        if changes
            .iter()
            .any(|change| change.key.as_str() == EDITOR_LOCALE_KEY)
        {
            if let Err(error) = self.i18n.synchronize_settings_snapshot(snapshot) {
                tracing::error!(%error, "validated editor locale could not be hot-applied");
            }
        }
    }
}

impl EditorMessageI18nEventSink {
    fn new(bus: SharedEditorMessageBus) -> Self {
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
    settings_store: Option<SettingsStore>,
}

impl EditorContextBuilder {
    pub fn new(scheduler: JobScheduler) -> Self {
        Self {
            bus: SharedEditorMessageBus::default(),
            scheduler,
            gateway: EditorRuntimeGatewayHandle::detached(),
            settings_store: None,
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

    pub(crate) fn with_settings_store(mut self, store: SettingsStore) -> Self {
        self.settings_store = Some(store);
        self
    }

    pub fn build(self) -> Arc<EditorContext> {
        let mut settings_registry = settings_registry_with_defaults();
        register_editor_job_quota_settings(&mut settings_registry)
            .expect("built-in editor job quota definitions are unique and valid");
        let settings_startup = match self.settings_store.as_ref() {
            Some(store) => SettingsStartup::load_from_store(settings_registry, store),
            None => SettingsStartup::load_from_environment(settings_registry),
        };
        report_user_layer_load(settings_startup.user_layer_load());
        let job_limits =
            resolve_editor_job_limits(settings_startup.registry(), self.scheduler.parallelism())
                .expect("registered and validated startup quotas resolve to editor job limits");
        let settings = Arc::new(settings_startup.into_authority());
        let events = Arc::new(EditorEventService::new(self.bus.clone()));
        let i18n = Arc::new(EditorI18nService::default());
        i18n.configure_event_sink(Arc::new(EditorMessageI18nEventSink::new(self.bus.clone())));
        let notifications = EditorNotificationService::default();
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            self.scheduler,
            self.bus.clone(),
            job_limits,
            notifications.job_progress_observer(),
        );
        let logs = Arc::new(EditorLogService::default());
        logs.configure_event_sink(Arc::new(EditorMessageLogEventSink::new(self.bus.clone())));
        let autosave = EditorAutosaveService::new(jobs.clone(), AutosavePolicy::default());
        let transactions = EditorTransactionEngine::with_event_sink(
            CoreEditContext::new(self.gateway.clone()),
            Arc::new(EditorMessageTransactionEventSink::new(self.bus.clone())),
        );
        let commands = EditorCommandRegistryHandle::default_workbench();
        let command_eval = CommandEvalSnapshotHandle::default();
        let tools = ToolSchedulerService::new(self.bus.clone());
        if let Err(error) = i18n.synchronize_user_locale(settings.as_ref()) {
            tracing::error!(%error, "persisted editor locale could not be applied at startup");
        }
        settings.configure_change_subscriber(Arc::new(EditorI18nSettingsChangeSubscriber {
            i18n: Arc::clone(&i18n),
        }));
        let settings_persistence = SettingsPersistenceService::new(Arc::clone(&settings));
        Arc::new(EditorContext::new(
            self.bus,
            events,
            i18n,
            jobs,
            logs,
            notifications,
            autosave,
            transactions,
            commands,
            command_eval,
            tools,
            settings,
            settings_persistence,
            self.gateway,
        ))
    }
}

fn report_user_layer_load(load: &SettingsUserLayerLoad) {
    match load {
        SettingsUserLayerLoad::Loaded {
            path,
            schema_version,
        } => tracing::info!(
            path = %path.display(),
            schema_version,
            "loaded editor User settings at startup"
        ),
        SettingsUserLayerLoad::Missing { path } => tracing::info!(
            path = %path.display(),
            "editor User settings are missing; using registered defaults"
        ),
        SettingsUserLayerLoad::Invalid { path, message } => tracing::warn!(
            path = ?path.as_deref(),
            error = %message,
            "editor User settings are invalid; using registered defaults"
        ),
    }
}

#[cfg(test)]
#[path = "builder/quota_startup_tests.rs"]
mod quota_startup_tests;

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::{
        EditorContextBuilder, EditorMessageTransactionEventSink, I18N_LOCALE_CHANGED_EVENT_SCHEMA,
        I18N_LOCALE_RESYNC_EVENT_SCHEMA, LOG_RECORD_EVENT_SCHEMA, LOG_RESYNC_EVENT_SCHEMA,
    };
    use crate::core::editing::engine::{
        HistoryContextId, TransactionEvent, TransactionEventKind, TransactionEventSink,
        TransactionId,
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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler()).build();

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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler()).build();
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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler()).build();
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
    fn log_service_publishes_sequence_notifications_to_the_canonical_log_topic() {
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler()).build();
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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler()).build();
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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler())
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
        let context = EditorContextBuilder::new(crate::core::jobs::test_job_scheduler())
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
        let log_sink = super::EditorMessageLogEventSink::new(bus.clone());
        let locale_sink = super::EditorMessageI18nEventSink::new(bus.clone());
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
        let log_sink = super::EditorMessageLogEventSink::new(bus.clone());
        let locale_sink = super::EditorMessageI18nEventSink::new(bus.clone());
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
}
