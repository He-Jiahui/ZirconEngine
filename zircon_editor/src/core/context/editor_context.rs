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
use crate::core::settings::{SettingsAuthority, SettingsPersistenceService};

use super::ToolSchedulerService;

/// Explicit L1 editor service aggregate. Each service owns its own synchronization.
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
    i18n: Arc<EditorI18nService>,
    jobs: EditorJobSystem,
    logs: EditorLogService,
    notifications: EditorNotificationService,
    transactions: Arc<EditorTransactionEngine>,
    dirty_documents: DirtyRegistry,
    commands: EditorCommandRegistryHandle,
    command_eval: CommandEvalSnapshotHandle,
    tools: ToolSchedulerService,
    settings: Arc<SettingsAuthority>,
    settings_persistence: SettingsPersistenceService,
    gateway: EditorRuntimeGatewayHandle,
}

impl EditorContext {
    pub(super) fn new(
        bus: SharedEditorMessageBus,
        events: Arc<EditorEventService>,
        i18n: Arc<EditorI18nService>,
        jobs: EditorJobSystem,
        logs: EditorLogService,
        notifications: EditorNotificationService,
        transactions: EditorTransactionEngine,
        commands: EditorCommandRegistryHandle,
        command_eval: CommandEvalSnapshotHandle,
        tools: ToolSchedulerService,
        settings: Arc<SettingsAuthority>,
        settings_persistence: SettingsPersistenceService,
        gateway: EditorRuntimeGatewayHandle,
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
            transactions,
            dirty_documents,
            commands,
            command_eval,
            tools,
            settings,
            settings_persistence,
            gateway,
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

    pub fn jobs(&self) -> &EditorJobSystem {
        &self.jobs
    }

    pub fn logs(&self) -> &EditorLogService {
        &self.logs
    }

    pub fn notifications(&self) -> &EditorNotificationService {
        &self.notifications
    }

    pub fn transactions(&self) -> &EditorTransactionEngine {
        self.transactions.as_ref()
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
        &self.settings
    }

    pub fn settings_persistence(&self) -> &SettingsPersistenceService {
        &self.settings_persistence
    }

    pub fn gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    pub fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.gateway.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::context::ToolSchedulerService;
    use crate::core::editor_message::{
        EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_TOOL, ToolMessage,
    };
    use crate::core::tools::{
        AcquireOutcome, ExclusiveResource, ToolId, ToolLifecycleEvent, ToolResourceSet,
    };

    #[test]
    fn tool_scheduler_service_publishes_typed_lifecycle_events_to_the_editor_bus() {
        let bus = SharedEditorMessageBus::default();
        let topic =
            EditorTopic::parse(TOPIC_TOOL).expect("the built-in tool topic should be valid");
        let subscriber = bus
            .register_subscriber([topic.clone()])
            .expect("the test subscriber should register");
        let scheduler = ToolSchedulerService::with_max_queue_per_resource(bus.clone(), 1);
        let holder = ToolId::parse("scene.mode").expect("the holder id should be valid");
        let queued = ToolId::parse("export.wizard").expect("the queued id should be valid");
        let denied = ToolId::parse("modal.asset-picker").expect("the denied id should be valid");

        assert_eq!(
            scheduler
                .acquire(holder.clone(), ExclusiveResource::ViewportInput)
                .outcome(),
            &AcquireOutcome::Acquired
        );
        assert_eq!(
            scheduler
                .acquire(queued.clone(), ExclusiveResource::ViewportInput)
                .outcome(),
            &AcquireOutcome::Queued { position: 1 }
        );
        let denied_report = scheduler.acquire(denied.clone(), ExclusiveResource::ViewportInput);
        assert!(matches!(
            denied_report.outcome(),
            AcquireOutcome::Denied { .. }
        ));
        let _ = scheduler.release(&holder, ExclusiveResource::ViewportInput);

        let deliveries = bus.drain_deliveries(subscriber);
        assert_eq!(deliveries.len(), 5);
        assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
        assert!(matches!(
            deliveries[0].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::Activated {
                tool,
                resource: ExclusiveResource::ViewportInput,
            })) if tool == &holder
        ));
        assert!(matches!(
            deliveries[2].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::Denied {
                tool,
                resource: ExclusiveResource::ViewportInput,
                ..
            })) if tool == &denied
        ));
        assert!(matches!(
            deliveries[3].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::Deactivated {
                tool,
                resource: ExclusiveResource::ViewportInput,
            })) if tool == &holder
        ));
        assert!(matches!(
            deliveries[4].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::Activated {
                tool,
                resource: ExclusiveResource::ViewportInput,
            })) if tool == &queued
        ));
    }

    #[test]
    fn tool_scheduler_service_publishes_atomic_set_lifecycle_events_after_unlocking() {
        let bus = SharedEditorMessageBus::default();
        let topic =
            EditorTopic::parse(TOPIC_TOOL).expect("the built-in tool topic should be valid");
        let subscriber = bus
            .register_subscriber([topic.clone()])
            .expect("the test subscriber should register");
        let scheduler = ToolSchedulerService::with_max_queue_per_resource(bus.clone(), 2);
        let scene = ToolId::parse("scene.viewport.mode").expect("the scene id should be valid");
        let export =
            ToolId::parse("workbench.build_export.windows").expect("the export id should be valid");
        let scene_resources = ToolResourceSet::new([
            ExclusiveResource::ViewportInput,
            ExclusiveResource::SceneModeSlot,
        ])
        .expect("the scene set should be valid");
        let export_resources = ToolResourceSet::new([
            ExclusiveResource::ViewportInput,
            ExclusiveResource::ModalSurface,
        ])
        .expect("the export set should be valid");

        let _ = scheduler.acquire_set(scene.clone(), scene_resources.clone());
        let _ = scheduler.acquire_set(export.clone(), export_resources.clone());
        let _ = scheduler.release_set(&scene, &scene_resources);

        let deliveries = bus.drain_deliveries(subscriber);
        assert_eq!(deliveries.len(), 4);
        assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
        assert!(matches!(
            deliveries[0].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::SetActivated {
                tool,
                resources,
            })) if tool == &scene && resources == &scene_resources
        ));
        assert!(matches!(
            deliveries[1].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::SetQueued {
                tool,
                resources,
                position: 1,
            })) if tool == &export && resources == &export_resources
        ));
        assert!(matches!(
            deliveries[2].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::SetDeactivated {
                tool,
                resources,
            })) if tool == &scene && resources == &scene_resources
        ));
        assert!(matches!(
            deliveries[3].message().payload(),
            EditorMessagePayload::Tool(ToolMessage::Lifecycle(ToolLifecycleEvent::SetActivated {
                tool,
                resources,
            })) if tool == &export && resources == &export_resources
        ));
    }
}
