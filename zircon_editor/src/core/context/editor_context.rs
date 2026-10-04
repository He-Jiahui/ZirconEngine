use std::sync::Arc;

use crate::core::asset::DirtyRegistry;
use crate::core::commands::{CommandEvalSnapshotHandle, EditorCommandRegistryHandle};
use crate::core::editing::engine::EditorTransactionEngine;
use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::gateway::{EditorRuntimeGatewayHandle, RuntimeCapabilities};
use crate::core::i18n::EditorI18nService;
use crate::core::jobs::EditorJobSystem;
use crate::core::logging::EditorLogService;
use crate::core::notifications::EditorNotificationService;
use crate::core::recovery::EditorAutosaveService;
use crate::core::settings::{SettingsAuthority, SettingsMutationCoordinator};

use super::ToolSchedulerService;

/// Explicit L1 editor service aggregate. Each service owns its own synchronization.
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
    i18n: Arc<EditorI18nService>,
    jobs: EditorJobSystem,
    logs: Arc<EditorLogService>,
    notifications: Arc<EditorNotificationService>,
    autosave: Arc<EditorAutosaveService>,
    transactions: Arc<EditorTransactionEngine>,
    dirty_documents: DirtyRegistry,
    commands: EditorCommandRegistryHandle,
    command_eval: CommandEvalSnapshotHandle,
    tools: ToolSchedulerService,
    settings_mutations: Arc<SettingsMutationCoordinator>,
    authoring_gateway: EditorRuntimeGatewayHandle,
    play_gateway: EditorRuntimeGatewayHandle,
}

impl EditorContext {
    pub(super) fn new(
        bus: SharedEditorMessageBus,
        events: Arc<EditorEventService>,
        i18n: Arc<EditorI18nService>,
        jobs: EditorJobSystem,
        logs: Arc<EditorLogService>,
        notifications: Arc<EditorNotificationService>,
        autosave: Arc<EditorAutosaveService>,
        transactions: EditorTransactionEngine,
        commands: EditorCommandRegistryHandle,
        command_eval: CommandEvalSnapshotHandle,
        tools: ToolSchedulerService,
        settings_mutations: Arc<SettingsMutationCoordinator>,
        authoring_gateway: EditorRuntimeGatewayHandle,
        play_gateway: EditorRuntimeGatewayHandle,
    ) -> Self {
        let transactions = Arc::new(transactions);
        let dirty_documents = DirtyRegistry::new(Arc::clone(&transactions));
        Self {
            bus,
            events,
            i18n,
            jobs,
            logs,
            notifications,
            autosave,
            transactions,
            dirty_documents,
            commands,
            command_eval,
            tools,
            settings_mutations,
            authoring_gateway,
            play_gateway,
        }
    }

    pub fn bus(&self) -> &SharedEditorMessageBus {
        &self.bus
    }

    pub fn events(&self) -> &Arc<EditorEventService> {
        &self.events
    }

    pub fn i18n(&self) -> &EditorI18nService {
        &self.i18n
    }

    pub(crate) fn i18n_handle(&self) -> Arc<EditorI18nService> {
        Arc::clone(&self.i18n)
    }

    pub fn jobs(&self) -> &EditorJobSystem {
        &self.jobs
    }

    pub fn logs(&self) -> &EditorLogService {
        self.logs.as_ref()
    }

    pub(crate) fn logs_handle(&self) -> Arc<EditorLogService> {
        Arc::clone(&self.logs)
    }

    pub fn notifications(&self) -> &EditorNotificationService {
        &self.notifications
    }

    pub(crate) fn autosave(&self) -> &EditorAutosaveService {
        self.autosave.as_ref()
    }

    pub fn transactions(&self) -> &EditorTransactionEngine {
        self.transactions.as_ref()
    }

    pub(crate) fn transactions_handle(&self) -> Arc<EditorTransactionEngine> {
        Arc::clone(&self.transactions)
    }

    pub fn dirty_documents(&self) -> &DirtyRegistry {
        &self.dirty_documents
    }

    pub fn commands(&self) -> &EditorCommandRegistryHandle {
        &self.commands
    }

    pub fn command_eval(&self) -> &CommandEvalSnapshotHandle {
        &self.command_eval
    }

    pub fn tools(&self) -> &ToolSchedulerService {
        &self.tools
    }

    pub fn settings(&self) -> &Arc<SettingsAuthority> {
        self.settings_mutations.authority()
    }

    pub fn settings_mutations(&self) -> &Arc<SettingsMutationCoordinator> {
        &self.settings_mutations
    }

    pub fn authoring_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.authoring_gateway
    }

    pub(crate) fn play_gateway_handle(&self) -> &EditorRuntimeGatewayHandle {
        &self.play_gateway
    }

    pub fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.authoring_gateway.capabilities()
    }
}

#[cfg(test)]
#[path = "tests/editor_context.rs"]
mod tests;
