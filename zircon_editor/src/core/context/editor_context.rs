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
mod tests {
    use crate::core::context::ToolSchedulerService;
    use crate::core::editor_event::ViewInstanceId;
    use crate::core::editor_message::{
        EditorMessageDelivery, EditorMessagePayload, EditorTopic, SharedEditorMessageBus,
        ToolMessage, TOPIC_TOOL,
    };
    use crate::core::tools::{
        AcquireOutcome, ToolInstanceId, ToolOwnerGeneration, ToolQueueLimits, ToolResourceKey,
        ToolResourceSet, ToolTransitionBatch,
    };
    use zircon_runtime_interface::ui::dispatch::UiWindowId;

    #[test]
    fn tool_scheduler_service_publishes_typed_lifecycle_events_to_the_editor_bus() {
        let bus = SharedEditorMessageBus::default();
        let topic =
            EditorTopic::parse(TOPIC_TOOL).expect("the built-in tool topic should be valid");
        let subscriber = bus
            .register_subscriber([topic.clone()])
            .expect("the test subscriber should register");
        let scheduler =
            ToolSchedulerService::with_queue_limits(bus.clone(), ToolQueueLimits::new(1, 1));
        let holder = ToolInstanceId::for_test("scene.mode", ToolOwnerGeneration::BUILTIN)
            .expect("the holder id should be valid");
        let queued = ToolInstanceId::for_test("export.wizard", ToolOwnerGeneration::BUILTIN)
            .expect("the queued id should be valid");
        let denied = ToolInstanceId::for_test("modal.asset-picker", ToolOwnerGeneration::BUILTIN)
            .expect("the denied id should be valid");

        let viewport_resource = viewport_resource();
        let resource_set = ToolResourceSet::single(viewport_resource.clone());
        let acquired_report = scheduler.acquire(holder, resource_set.clone()).unwrap();
        let AcquireOutcome::Acquired {
            lease: holder_lease,
        } = acquired_report.outcome()
        else {
            panic!("the first tool should acquire the viewport");
        };
        let holder_lease = holder_lease.clone();
        let acquired_events = acquired_report.events().to_vec();
        let queued_report = scheduler.acquire(queued, resource_set.clone()).unwrap();
        let AcquireOutcome::Queued {
            request: queued_request,
            position: 1,
        } = queued_report.outcome()
        else {
            panic!("the second tool should queue");
        };
        let queued_request = queued_request.clone();
        let queued_events = queued_report.events().to_vec();
        let denied_report = scheduler.acquire(denied, resource_set).unwrap();
        assert!(matches!(
            denied_report.outcome(),
            AcquireOutcome::Denied { .. }
        ));
        let denied_events = denied_report.events().to_vec();
        let released_report = scheduler.release(holder_lease.id()).unwrap();
        let released_events = released_report.events().to_vec();

        let deliveries = bus.drain_deliveries(subscriber);
        assert_eq!(deliveries.len(), 4);
        assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
        let batches = deliveries.iter().map(transition_batch).collect::<Vec<_>>();
        assert_eq!(
            batches
                .iter()
                .map(|batch| batch.revision().value())
                .collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
        assert_eq!(batches[0].events(), acquired_events);
        assert_eq!(batches[1].events(), queued_events);
        assert_eq!(batches[2].events(), denied_events);
        assert_eq!(batches[3].events(), released_events);
        assert_eq!(
            scheduler
                .holder(&viewport_resource)
                .map(|lease| lease.request_id()),
            Some(queued_request.id())
        );
        let health = scheduler.delivery_health();
        assert_eq!(health.committed_revision().value(), 4);
        assert_eq!(health.dispatched_revision().value(), 4);
        assert_eq!(health.delivered_batches(), 4);
        assert_eq!(health.unobserved_batches(), 0);
        assert!(!health.requires_resync());
    }

    fn transition_batch(delivery: &EditorMessageDelivery) -> &ToolTransitionBatch {
        match delivery.message().payload() {
            EditorMessagePayload::Tool(ToolMessage::Transition(batch)) => batch,
            payload => panic!("expected a tool transition batch, got {payload:?}"),
        }
    }

    #[test]
    fn tool_scheduler_service_publishes_atomic_set_transition_batches() {
        let bus = SharedEditorMessageBus::default();
        let topic =
            EditorTopic::parse(TOPIC_TOOL).expect("the built-in tool topic should be valid");
        let subscriber = bus
            .register_subscriber([topic.clone()])
            .expect("the test subscriber should register");
        let scheduler =
            ToolSchedulerService::with_queue_limits(bus.clone(), ToolQueueLimits::new(2, 2));
        let scene = ToolInstanceId::for_test("scene.viewport.mode", ToolOwnerGeneration::BUILTIN)
            .expect("the scene id should be valid");
        let export = ToolInstanceId::for_test(
            "workbench.build_export.windows",
            ToolOwnerGeneration::BUILTIN,
        )
        .expect("the export id should be valid");
        let scene_resources = ToolResourceSet::new([
            viewport_resource(),
            ToolResourceKey::scene_mode_slot(ViewInstanceId::new("editor.scene#1")),
        ])
        .expect("the scene set should be valid");
        let export_resources = ToolResourceSet::new([viewport_resource(), modal_resource()])
            .expect("the export set should be valid");

        let scene_report = scheduler.acquire(scene, scene_resources).unwrap();
        let AcquireOutcome::Acquired { lease: scene_lease } = scene_report.outcome() else {
            panic!("scene resources should acquire atomically");
        };
        let scene_lease = scene_lease.clone();
        let scene_events = scene_report.events().to_vec();
        let export_report = scheduler.acquire(export, export_resources).unwrap();
        let AcquireOutcome::Queued {
            request: export_request,
            ..
        } = export_report.outcome()
        else {
            panic!("export resources should queue atomically");
        };
        let export_request = export_request.clone();
        let export_events = export_report.events().to_vec();
        let release_report = scheduler.release(scene_lease.id()).unwrap();
        let release_events = release_report.events().to_vec();

        let deliveries = bus.drain_deliveries(subscriber);
        assert_eq!(deliveries.len(), 3);
        assert!(deliveries.iter().all(|delivery| delivery.topic() == &topic));
        let batches = deliveries.iter().map(transition_batch).collect::<Vec<_>>();
        assert_eq!(
            batches
                .iter()
                .map(|batch| batch.revision().value())
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(batches[0].events(), scene_events);
        assert_eq!(batches[1].events(), export_events);
        assert_eq!(batches[2].events(), release_events);
        assert_eq!(
            scheduler
                .holder(&modal_resource())
                .map(|lease| lease.request_id()),
            Some(export_request.id())
        );
    }

    fn viewport_resource() -> ToolResourceKey {
        ToolResourceKey::viewport_input(ViewInstanceId::new("editor.scene#1"))
    }

    fn modal_resource() -> ToolResourceKey {
        ToolResourceKey::modal_surface(UiWindowId::new("editor.main"))
    }
}
