use std::sync::Arc;

use crate::core::commands::{CommandEvalSnapshotHandle, EditorCommandRegistryHandle};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::EditorTransactionEngine;
use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::i18n::EditorI18nService;
use crate::core::jobs::{
    register_editor_job_quota_settings, resolve_editor_job_limits, EditorJobSystem,
};
use crate::core::logging::EditorLogService;
use crate::core::notifications::EditorNotificationService;
use crate::core::recovery::{AutosavePolicy, EditorAutosaveService};
use crate::core::settings::{
    settings_registry_with_defaults, SettingsMutationCoordinator, SettingsPersistenceService,
    SettingsStartup, SettingsStore, SettingsUserLayerLoad,
};
use zircon_runtime::core::runtime::tasks::JobScheduler;

use super::{EditorContext, ToolSchedulerService};

mod event_sinks;
mod settings_locale_sync;
mod settings_persistence_health;

use event_sinks::{
    EditorMessageI18nEventSink, EditorMessageLogEventSink, EditorMessageTransactionEventSink,
};
use settings_locale_sync::EditorSettingsChangeSubscriber;
use settings_persistence_health::EditorSettingsPersistenceHealthSubscriber;

pub struct EditorContextBuilder {
    bus: SharedEditorMessageBus,
    scheduler: JobScheduler,
    settings_io_scheduler: JobScheduler,
    authoring_gateway: EditorRuntimeGatewayHandle,
    settings_store: Option<SettingsStore>,
}

impl EditorContextBuilder {
    pub fn new(scheduler: JobScheduler, settings_io_scheduler: JobScheduler) -> Self {
        Self {
            bus: SharedEditorMessageBus::default(),
            scheduler,
            settings_io_scheduler,
            authoring_gateway: EditorRuntimeGatewayHandle::detached(),
            settings_store: None,
        }
    }

    pub fn with_bus(mut self, bus: SharedEditorMessageBus) -> Self {
        self.bus = bus;
        self
    }

    pub fn with_authoring_gateway(mut self, authoring_gateway: EditorRuntimeGatewayHandle) -> Self {
        self.authoring_gateway = authoring_gateway;
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
        let settings_store = self
            .settings_store
            .or_else(|| SettingsStore::from_user_environment().ok());
        let settings_startup = match settings_store.as_ref() {
            Some(store) => SettingsStartup::load_from_store(settings_registry, store),
            None => SettingsStartup::load_from_environment(settings_registry),
        };
        report_user_layer_load(settings_startup.user_layer_load());
        let writable_user_store = (!matches!(
            settings_startup.user_layer_load(),
            SettingsUserLayerLoad::Invalid { .. }
        ))
        .then(|| settings_store.clone())
        .flatten();
        let job_limits =
            resolve_editor_job_limits(settings_startup.registry(), self.scheduler.parallelism())
                .expect("registered and validated startup quotas resolve to editor job limits");
        let settings = Arc::new(settings_startup.into_authority());
        let events = Arc::new(EditorEventService::new(self.bus.clone()));
        let i18n = Arc::new(EditorI18nService::default());
        i18n.configure_event_sink(Arc::new(EditorMessageI18nEventSink::new(self.bus.clone())));
        let notifications = Arc::new(EditorNotificationService::default());
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            self.scheduler,
            self.bus.clone(),
            job_limits,
            notifications.job_progress_observer(),
        );
        let logs = Arc::new(EditorLogService::default());
        logs.configure_event_sink(Arc::new(EditorMessageLogEventSink::new(self.bus.clone())));
        let autosave_policy = AutosavePolicy::new(settings.snapshot().autosave_interval())
            .expect("the validated autosave setting must produce a non-zero policy");
        let autosave = Arc::new(EditorAutosaveService::new(jobs.clone(), autosave_policy));
        let play_gateway = EditorRuntimeGatewayHandle::detached();
        let transactions = EditorTransactionEngine::with_event_sink(
            CoreEditContext::with_world_gateways(
                self.authoring_gateway.clone(),
                play_gateway.clone(),
            ),
            Arc::new(EditorMessageTransactionEventSink::new(self.bus.clone())),
        );
        let commands = EditorCommandRegistryHandle::default_workbench();
        let command_eval = CommandEvalSnapshotHandle::default();
        let tools = ToolSchedulerService::new(self.bus.clone());
        if let Err(error) = i18n.synchronize_user_locale(settings.as_ref()) {
            tracing::error!(%error, "persisted editor locale could not be applied at startup");
        }
        settings.configure_change_subscriber(Arc::new(EditorSettingsChangeSubscriber {
            i18n: Arc::clone(&i18n),
            autosave: Arc::clone(&autosave),
        }));
        let settings_persistence =
            SettingsPersistenceService::new(Arc::clone(&settings), self.settings_io_scheduler);
        let settings_mutations = Arc::new(SettingsMutationCoordinator::new(
            Arc::clone(&settings),
            settings_persistence,
            writable_user_store,
        ));
        settings_mutations.configure_persistence_health_subscriber(Arc::new(
            EditorSettingsPersistenceHealthSubscriber::new(Arc::clone(&notifications)),
        ));
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
            settings_mutations,
            self.authoring_gateway,
            play_gateway,
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
mod tests;
