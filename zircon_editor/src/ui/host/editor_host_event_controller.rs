use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use crate::core::commands::{EditorCommandPaletteMru, EditorCommandRegistryHandle};
use crate::core::context::EditorContext;
use crate::core::editor_message::{EditorSubscriberId, EditorTopic, TOPIC_SCENE_INSPECTION};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::gateway::EditorRuntimeGatewayHandle;
#[cfg(test)]
use crate::core::gateway::SharedEditorRuntimeGateway;
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::core::play::{
    PlayInstanceId, PlaySessionController, SharedPlayBackend, SharedPluginBridgeActivation,
    WorldDomain,
};
#[cfg(test)]
use crate::core::play::{PlayKind, PlaySessionError, PlayStartRequest, TestAttachablePlayBackend};
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerHost;
use crate::core::sync::WorldSyncPump;
use crate::ui::workbench::shell_state::WorkbenchShellState;
use crate::ui::workbench::state::EditorState;

use super::play_hierarchy_projection::PlayHierarchyProjection;
use super::play_inspector_projection::PlayInspectorProjection;
use super::play_pending_decision::PlayPendingEditDecisionAdapter;
use super::scene_inspection_publication::SceneInspectionPublication;
use super::EditorManager;

const FIRST_PLAY_SESSION_GENERATION: u64 = 1;
const UNKNOWN_PLAY_BACKEND_LOG_FRAME: u64 = 0;

mod play_gizmo;
mod play_hierarchy;
mod play_inspector;
mod play_preview_input;
mod play_viewport_pick;
mod play_world_replacement;
mod runtime_event_consumers;
mod runtime_shutdown;
mod simulate_camera;

pub(crate) use play_gizmo::{PlayGizmoOverlaySnapshot, PlayGizmoPointerOutcome};

pub use runtime_shutdown::{
    EditorPlaySessionShutdownReceipt, EditorPlayStateShutdownDisposition,
    EditorRuntimeSessionShutdownReceipt, EditorTerminalPlayDetachError,
    RuntimeEventConsumerShutdownDisposition, RuntimePlayBackendRetirementDisposition,
    RuntimePlayGatewayShutdownDisposition, RuntimePlaySessionShutdownDisposition,
};

/// UI host coordinator over independently synchronized editor owners.
pub struct EditorHostEventController {
    context: Arc<EditorContext>,
    shell: Arc<WorkbenchShellState>,
    commands: EditorCommandRegistryHandle,
    play_sessions: Arc<PlaySessionController>,
    play_pending_decisions: PlayPendingEditDecisionAdapter,
    pub(super) scene_inspection_publication: Mutex<SceneInspectionPublication>,
    pub(super) retained_scene_inspection_subscriber: EditorSubscriberId,
    pub(super) edit_world_sync: Mutex<WorldSyncPump>,
    pub(super) play_world_sync: Mutex<WorldSyncPump>,
    pub(super) play_hierarchy_projection: Mutex<PlayHierarchyProjection>,
    pub(super) play_inspector_projection: Mutex<PlayInspectorProjection>,
    pub(super) play_gizmo: Mutex<play_gizmo::PlayGizmoInteractionController>,
    pub(super) runtime_event_consumers: EditorRuntimeEventConsumerHost,
    pub(super) plugin_registration_gate: Mutex<()>,
    next_play_session_generation: AtomicU64,
}

impl EditorHostEventController {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let context = manager.context().clone();
        let commands = context.commands().clone();
        let play_sessions = Arc::new(PlaySessionController::with_message_bus_and_play_gateway(
            context.bus().clone(),
            context.play_gateway_handle().clone(),
        ));
        let retained_scene_inspection_subscriber = context
            .bus()
            .register_subscriber([EditorTopic::parse(TOPIC_SCENE_INSPECTION)
                .expect("scene-inspection topic is a static editor protocol invariant")])
            .expect("retained scene-inspection subscriber must register during host construction");
        let controller = Self {
            context: context.clone(),
            shell: Arc::new(WorkbenchShellState::new(state, Arc::clone(&manager))),
            commands,
            play_sessions: play_sessions.clone(),
            play_pending_decisions: PlayPendingEditDecisionAdapter::default(),
            scene_inspection_publication: Mutex::new(SceneInspectionPublication::default()),
            retained_scene_inspection_subscriber,
            edit_world_sync: Mutex::new(WorldSyncPump::default()),
            play_world_sync: Mutex::new(WorldSyncPump::default()),
            play_hierarchy_projection: Mutex::new(PlayHierarchyProjection::default()),
            play_inspector_projection: Mutex::new(PlayInspectorProjection::default()),
            play_gizmo: Mutex::new(play_gizmo::PlayGizmoInteractionController::default()),
            runtime_event_consumers: EditorRuntimeEventConsumerHost::new(
                play_sessions.play_gateway_handle(),
            ),
            plugin_registration_gate: Mutex::new(()),
            next_play_session_generation: AtomicU64::new(FIRST_PLAY_SESSION_GENERATION),
        };
        controller.seed_scene_inspection_publication();
        controller.refresh_reflection();
        controller
    }

    pub fn context(&self) -> &Arc<EditorContext> {
        &self.context
    }

    /// Pumps plugin lifecycle subscriptions outside the workbench shell lock.
    pub fn pump_plugin_lifecycle_messages(&self) -> Result<usize, String> {
        let manager = { Arc::clone(&self.shell.lock().manager) };
        manager.pump_plugin_lifecycle_messages()
    }

    pub fn set_plugin_bridge_activation(&self, activation: SharedPluginBridgeActivation) {
        self.play_sessions.set_plugin_activation(activation);
    }

    pub fn set_play_backend(&self, backend: SharedPlayBackend) {
        self.play_sessions.set_play_backend(backend);
    }

    #[cfg(test)]
    pub(crate) fn start_test_play_gateway(
        &self,
        kind: PlayKind,
        gateway: SharedEditorRuntimeGateway,
    ) -> Result<PlayInstanceId, PlaySessionError> {
        self.set_play_backend(Arc::new(TestAttachablePlayBackend::new(gateway)));
        self.play_sessions
            .request_play(PlayStartRequest::immediate(kind, None))?;
        match self.play_sessions.attached_world_domain() {
            Some(WorldDomain::Play(instance)) => Ok(instance),
            _ => Err(PlaySessionError::InvalidTransition {
                mode: self.play_sessions.mode(),
                event: "test_backend_started_without_gateway_attachment",
            }),
        }
    }

    pub fn gateway_for(&self, domain: WorldDomain) -> Option<EditorRuntimeGatewayHandle> {
        match domain {
            WorldDomain::Edit => Some(self.context.authoring_gateway().clone()),
            WorldDomain::Play(instance) => self.play_sessions.play_gateway(instance),
        }
    }

    pub(crate) fn shell(&self) -> &WorkbenchShellState {
        &self.shell
    }

    pub(in crate::ui::host) fn play_pending_decisions(&self) -> &PlayPendingEditDecisionAdapter {
        &self.play_pending_decisions
    }

    pub(crate) fn commands(&self) -> &EditorCommandRegistryHandle {
        &self.commands
    }

    /// Reads the current authority-derived keymap without retaining a controller copy.
    pub(crate) fn keymap(&self) -> crate::core::commands::EditorKeymap {
        self.shell.lock().manager.keymap()
    }

    pub(crate) fn command_palette_mru(&self) -> EditorCommandPaletteMru {
        self.shell.lock().manager.command_palette_mru()
    }

    pub(crate) fn record_command_palette_usage(&self, command: EditorOperationPath) {
        self.shell
            .lock()
            .manager
            .record_command_palette_usage(command);
    }

    pub(crate) fn play_sessions(&self) -> &PlaySessionController {
        &self.play_sessions
    }

    pub(in crate::ui::host) fn log_play_backend_diagnostics(&self, diagnostics: &[String]) {
        let source = play_backend_log_source(&self.play_sessions);
        emit_play_backend_diagnostics(self.context.logs(), &source, diagnostics);
    }
}

impl Drop for EditorHostEventController {
    fn drop(&mut self) {
        self.context
            .bus()
            .unregister_subscriber(self.retained_scene_inspection_subscriber);
    }
}

fn play_backend_log_source(play_sessions: &PlaySessionController) -> LogSource {
    match play_sessions.attached_world_domain() {
        Some(WorldDomain::Play(instance)) => LogSource::play(instance),
        Some(WorldDomain::Edit) | None => LogSource::runtime(),
    }
}

fn emit_play_backend_diagnostics(
    logs: &EditorLogService,
    source: &LogSource,
    diagnostics: &[String],
) {
    for diagnostic in diagnostics {
        if diagnostic.trim().is_empty() {
            continue;
        }
        let severity = play_backend_diagnostic_severity(diagnostic);
        let source_label = play_backend_diagnostic_source_label(diagnostic);
        let entry = LogEntry::new(
            source.clone(),
            severity,
            diagnostic.clone(),
            UNKNOWN_PLAY_BACKEND_LOG_FRAME,
            None,
        )
        .or_else(|_| {
            LogEntry::new(
                source.clone(),
                severity,
                format!(
                    "play_backend_output source={source_label} diagnostic exceeds the log-entry limit."
                ),
                UNKNOWN_PLAY_BACKEND_LOG_FRAME,
                None,
            )
        });
        if let Ok(entry) = entry {
            let _ = logs.emit(entry);
        }
    }
}

fn play_backend_diagnostic_severity(diagnostic: &str) -> LogSeverity {
    if diagnostic.starts_with("process.stderr:") || diagnostic.starts_with("process.output") {
        LogSeverity::Warning
    } else {
        LogSeverity::Info
    }
}

fn play_backend_diagnostic_source_label(diagnostic: &str) -> &str {
    diagnostic
        .split_once(':')
        .map_or("process.output", |(label, _)| label)
}

#[cfg(test)]
mod lifecycle_contract_tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
    use zircon_runtime_interface::math::UVec2;
    use zircon_runtime_interface::{
        GatewaySessionIdentity, ZrRuntimeAllocationId, ZrRuntimeApiV8, ZrRuntimeOperationHandle,
        ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
        ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
        ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
        ZrStatus,
    };

    use crate::core::editing::command::EditorCommand;
    use crate::core::editing::engine::HistoryContextId;
    use crate::core::editing::intent::EditorIntent;
    use crate::core::gateway::{
        DetachedEditorRuntimeGateway, EditorRuntimeGateway, GatewayError, InProcessGateway,
        RuntimeCapabilities, SessionGateway,
    };
    use crate::core::logging::{EditorLogService, LogFilter, LogSource};
    use crate::core::play::{
        PlayBackend, PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure,
        PlayBackendStartReport, PlayBackendStopReport, PlayKind, PlayModeKind,
        PlaySessionController, PlayStartRequest, PluginBridgeActivation,
        PluginBridgeActivationReport, WorldDomain,
    };
    use crate::core::runtime_event_consumer::{
        EditorRuntimeEventConsumerRegistration, EditorRuntimeEventConsumerRegistry,
        EditorRuntimeEventConsumerState,
    };
    use crate::ui::workbench::state::EditorState;
    use zircon_runtime::plugin::PluginEventConsumerManifest;

    use super::{
        emit_play_backend_diagnostics, play_backend_log_source, EditorHostEventController,
        EditorManager,
    };

    unsafe extern "C" fn release_test_allocation(
        _session: ZrRuntimeSessionHandle,
        _allocation: ZrRuntimeAllocationId,
    ) -> ZrStatus {
        ZrStatus::ok()
    }

    unsafe extern "C" fn request_test_viewport_pick(
        _session: ZrRuntimeSessionHandle,
        _request: ZrRuntimeViewportPickRequestV1,
        _out_ticket: *mut ZrRuntimeViewportPickTicket,
    ) -> ZrStatus {
        ZrStatus::ok()
    }

    unsafe extern "C" fn poll_test_viewport_pick(
        _session: ZrRuntimeSessionHandle,
        _ticket: ZrRuntimeViewportPickTicket,
        _out_result: *mut ZrRuntimeViewportPickResultV1,
    ) -> ZrStatus {
        ZrStatus::ok()
    }

    unsafe extern "C" fn cancel_test_viewport_pick(
        _session: ZrRuntimeSessionHandle,
        _ticket: ZrRuntimeViewportPickTicket,
    ) -> ZrStatus {
        ZrStatus::ok()
    }

    fn test_runtime_api() -> ZrRuntimeApiV8 {
        let mut api = ZrRuntimeApiV8::empty();
        api.release_allocation = Some(release_test_allocation);
        api.request_viewport_pick = Some(request_test_viewport_pick);
        api.poll_viewport_pick = Some(poll_test_viewport_pick);
        api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
        api
    }

    struct RefusingUnsubscribeGateway;

    impl EditorRuntimeGateway for RefusingUnsubscribeGateway {
        fn session_handle(&self) -> ZrRuntimeSessionHandle {
            ZrRuntimeSessionHandle::new(71)
        }

        fn session_identity(&self) -> GatewaySessionIdentity {
            GatewaySessionIdentity::new(17, self.session_handle(), 23, None)
        }

        fn subscribe_plugin_event(
            &self,
            _event_id: &str,
            _payload_schema: &str,
        ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
            Ok(Some(ZrRuntimePluginEventSubscriptionHandle::new(29)))
        }

        fn unsubscribe_plugin_event(
            &self,
            _subscription: ZrRuntimePluginEventSubscriptionHandle,
        ) -> Result<bool, GatewayError> {
            Ok(false)
        }

        fn submit_operation(
            &self,
            _request: ZrRuntimeOperationSubmitRequestV1,
        ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.submit",
            })
        }

        fn poll_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.poll",
            })
        }

        fn harvest_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.harvest",
            })
        }
    }

    struct NoopConsumer;

    impl EditorRuntimeEventConsumerState for NoopConsumer {
        type Payload = serde_json::Value;
        type Error = Infallible;

        fn begin_session(&mut self, _play_session_id: u64) {}

        fn consume(
            &mut self,
            _play_session_id: u64,
            _sequence: u64,
            _payload: Self::Payload,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn end_session(&mut self, _play_session_id: u64) {}
    }

    struct RefusingStopBackend;

    impl PlayBackend for RefusingStopBackend {
        fn start(
            &self,
            _request: &PlayStartRequest,
        ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
            Ok(PlayBackendStartReport::with_gateway(
                Vec::new(),
                Arc::new(DetachedEditorRuntimeGateway),
            ))
        }

        fn stop(&self) -> Result<PlayBackendStopReport, String> {
            Err("test backend refused explicit shutdown".to_string())
        }

        fn retire(&self) -> Result<PlayBackendRetireReport, String> {
            Ok(PlayBackendRetireReport::default())
        }

        fn poll(&self) -> Result<PlayBackendPoll, String> {
            Ok(PlayBackendPoll::Running {
                diagnostics: Vec::new(),
            })
        }
    }

    struct RefusingPluginDeactivation;

    impl PluginBridgeActivation for RefusingPluginDeactivation {
        fn activate(
            &self,
            _project_root: Option<&std::path::Path>,
        ) -> Result<PluginBridgeActivationReport, String> {
            Ok(PluginBridgeActivationReport::default())
        }

        fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
            Err("test plugin restoration requires retry".to_string())
        }
    }

    #[test]
    fn runtime_event_consumer_orchestration_stays_in_its_named_owner() {
        let root = include_str!("editor_host_event_controller.rs");
        let production = root
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("controller lifecycle tests should remain separate from production code");
        let owner = include_str!("editor_host_event_controller/runtime_event_consumers.rs");

        assert!(root.lines().count() <= 1_000);
        assert!(production.contains("mod runtime_event_consumers;"));
        for method in [
            "register_runtime_event_consumers",
            "begin_runtime_event_consumers",
            "pump_runtime_event_consumers",
            "end_runtime_event_consumers",
            "runtime_event_consumer_session_active",
        ] {
            assert!(!production.contains(&format!("fn {method}(")));
            assert!(owner.contains(&format!("fn {method}(")));
        }
    }

    #[test]
    fn retained_hierarchy_transport_resources_share_the_host_controller_lifetime() {
        let source = include_str!("editor_host_event_controller.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("controller lifecycle tests should remain separate from production code");

        assert!(production.contains("retained_scene_inspection_subscriber: EditorSubscriberId"));
        assert!(production.contains("edit_world_sync: Mutex<WorldSyncPump>"));
        assert!(production.contains("play_world_sync: Mutex<WorldSyncPump>"));
        assert!(production.contains("register_subscriber"));
        assert!(production.contains("impl Drop for EditorHostEventController"));
        assert!(production.contains("unregister_subscriber"));
    }

    #[test]
    fn unattached_play_backend_output_enters_the_runtime_log_channel() {
        let play_sessions = PlaySessionController::new();
        let logs = EditorLogService::default();
        let source = play_backend_log_source(&play_sessions);

        emit_play_backend_diagnostics(
            &logs,
            &source,
            &["process.stdout: runtime ready".to_string()],
        );

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].entry().source(), &LogSource::runtime());
    }

    #[test]
    fn attached_play_backend_output_enters_its_play_instance_log_channel() {
        let play_sessions = PlaySessionController::new();
        play_sessions.set_play_backend(Arc::new(TestAttachablePlayBackend::new(Arc::new(
            DetachedEditorRuntimeGateway,
        ))));
        play_sessions
            .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
            .expect("the attachable test backend should start");
        let Some(WorldDomain::Play(instance)) = play_sessions.attached_world_domain() else {
            panic!("the attachable test backend should publish a Play identity");
        };
        let logs = EditorLogService::default();
        let source = play_backend_log_source(&play_sessions);

        emit_play_backend_diagnostics(&logs, &source, &["process.stdout: play ready".to_string()]);

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].entry().source(), &LogSource::play(instance));
    }

    #[test]
    fn explicit_runtime_shutdown_retires_the_current_play_link_without_drop() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
            .expect("a detached test gateway should attach to the play domain");

        let receipt = controller.shutdown_runtime_session();

        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::Stopped { report }
                if report.changed && report.mode == PlayModeKind::Edit
        ));
        assert!(matches!(
            receipt.event_consumers(),
            super::RuntimeEventConsumerShutdownDisposition::NotActive
        ));
        assert!(receipt.edit_world_sync().watches().is_empty());
        assert!(receipt.play_world_sync().watches().is_empty());
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());
    }

    #[test]
    fn explicit_runtime_shutdown_discards_play_history_before_detach() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_level = DefaultLevelManager::default().create_default_level();
        let play_instance = controller
            .start_test_play_gateway(
                PlayKind::Play,
                Arc::new(InProcessGateway::for_authoring_level(play_level)),
            )
            .expect("the test play world should attach");
        let history = HistoryContextId::PlaySession(play_instance);
        let mut scope = controller
            .context()
            .transactions()
            .begin("create runtime node", history)
            .expect("the attached play world should accept its history");
        scope
            .push(EditorCommand::create_node(NodeKind::Cube))
            .expect("the play command should target the attached play world");
        scope.commit().expect("the play command should commit");
        assert_eq!(
            controller
                .context()
                .transactions()
                .history_status(history)
                .unwrap()
                .len,
            1
        );
        let receipt = controller.shutdown_runtime_session();

        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert_eq!(
            controller
                .context()
                .transactions()
                .history_status(history)
                .unwrap()
                .len,
            0
        );
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());
    }

    #[test]
    fn explicit_runtime_shutdown_stops_play_before_detaching_the_gateway() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
            .expect("a detached test gateway should attach to the play domain");

        let receipt = controller.shutdown_runtime_session();

        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::Stopped { report }
                if report.changed && report.mode == PlayModeKind::Edit
        ));
        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert_eq!(controller.play_sessions().mode(), PlayModeKind::Edit);
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());
    }

    #[test]
    fn explicit_runtime_shutdown_detaches_a_stopped_play_link_with_retryable_plugin_cleanup() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        controller.set_plugin_activation(Arc::new(RefusingPluginDeactivation));
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
            .expect("a detached test gateway should attach to the play domain");

        let receipt = controller.shutdown_runtime_session();

        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::StoppedWithCleanupFailure { .. }
        ));
        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert_eq!(
            controller.play_sessions().mode(),
            PlayModeKind::CleanupFailed
        );
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());
    }

    #[test]
    fn project_close_play_shutdown_restores_edit_state_and_preserves_edit_gateway() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
            .expect("a detached test gateway should attach to the play domain");
        controller
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("the retained shell should enter play before project close");

        let receipt = controller.shutdown_play_session_for_project_close();

        assert!(receipt.is_ready_for_project_close());
        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::Stopped { report }
                if report.changed && report.mode == PlayModeKind::Edit
        ));
        assert!(matches!(
            receipt.editor_state(),
            super::EditorPlayStateShutdownDisposition::Restored
        ));
        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert!(!controller.shell().lock().state.is_playing());
        assert!(controller.gateway_for(WorldDomain::Edit).is_some());
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());
    }

    #[test]
    fn project_close_play_shutdown_blocks_closeout_until_plugin_cleanup_is_repaired() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        controller.set_plugin_activation(Arc::new(RefusingPluginDeactivation));
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
            .expect("a detached test gateway should attach to the play domain");
        controller
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("the retained shell should enter play before project close");

        let receipt = controller.shutdown_play_session_for_project_close();

        assert!(!receipt.is_ready_for_project_close());
        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::StoppedWithCleanupFailure { .. }
        ));
        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::Detached { instance, .. }
                if *instance == play_instance
        ));
        assert!(matches!(
            receipt.editor_state(),
            super::EditorPlayStateShutdownDisposition::Restored
        ));
    }

    #[test]
    fn explicit_runtime_shutdown_keeps_the_play_link_when_backend_stop_is_deferred() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        controller.set_play_backend(Arc::new(RefusingStopBackend));
        controller
            .play_sessions()
            .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
            .expect("the test play session should start");
        let Some(WorldDomain::Play(play_instance)) =
            controller.play_sessions().attached_world_domain()
        else {
            panic!("the refusing backend should publish its Play gateway");
        };

        let receipt = controller.shutdown_runtime_session();

        assert!(matches!(
            receipt.play_session(),
            super::RuntimePlaySessionShutdownDisposition::RetirementDeferred {
                mode: PlayModeKind::Playing,
                ..
            }
        ));
        assert!(matches!(
            receipt.play_gateway(),
            super::RuntimePlayGatewayShutdownDisposition::RetainedForActivePlay {
                mode: PlayModeKind::Playing
            }
        ));
        assert_eq!(controller.play_sessions().mode(), PlayModeKind::Playing);
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_some());
    }

    #[test]
    fn consumer_remote_cleanup_failure_is_locally_terminal_for_play_exit() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(RefusingUnsubscribeGateway))
            .expect("the test play gateway should attach");
        assert_eq!(controller.play_sessions().mode(), PlayModeKind::Playing);
        let mut registry = EditorRuntimeEventConsumerRegistry::default();
        registry
            .register(EditorRuntimeEventConsumerRegistration::typed(
                PluginEventConsumerManifest::new(
                    "tests.local_retirement",
                    "tests.local_retirement.event",
                    "tests.local_retirement.v1",
                ),
                Arc::new(std::sync::Mutex::new(NoopConsumer)),
            ))
            .expect("the test runtime event consumer should register");
        controller
            .register_runtime_event_consumers(registry)
            .expect("the controller should accept the test consumer");
        controller
            .begin_runtime_event_consumers()
            .expect("the controller should begin the test consumer");

        let disposition = controller.shutdown_runtime_event_consumers();

        assert!(matches!(
            disposition,
            super::RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { .. }
        ));
        assert!(!controller.runtime_event_consumer_session_active());
        assert_eq!(
            controller.runtime_event_consumers.active_consumer_count(),
            0
        );
        let stop_transition = controller
            .play_sessions()
            .request_stop()
            .expect("local consumer retirement must not block play shutdown");
        assert!(stop_transition.changed);
        assert_eq!(stop_transition.mode, PlayModeKind::Edit);
    }

    #[test]
    fn host_play_attachment_preserves_edit_selection_and_undo_history() {
        let core = CoreRuntime::new();
        let manager = Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let authoring_level = DefaultLevelManager::default().create_default_level();
        let state = EditorState::with_default_selection_with_context(
            authoring_level.clone(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let selection_before = controller
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .expect("the default authoring level should select a node");

        let created = {
            let mut shell = controller.shell().lock();
            assert!(shell
                .state
                .apply_intent(EditorIntent::CreateNode(NodeKind::Empty))
                .expect("the edit domain should accept a scene command"));
            shell
                .state
                .viewport_controller
                .selection()
                .active_primary()
                .expect("the created node should be selected")
        };

        let session_gateway = unsafe {
            SessionGateway::new_with_identity(
                Arc::new(()),
                test_runtime_api(),
                ZrRuntimeSessionHandle::new(2),
                zircon_runtime_interface::GatewaySessionIdentity::new(
                    2,
                    ZrRuntimeSessionHandle::new(2),
                    1,
                    None,
                ),
                RuntimeCapabilities::editor_default(),
                Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
            )
        }
        .expect("a valid session handle should construct a serialized gateway");
        let play_instance = controller
            .start_test_play_gateway(PlayKind::Play, Arc::new(session_gateway))
            .expect("the host should attach only the play-domain gateway");

        let edit_gateway = controller
            .gateway_for(WorldDomain::Edit)
            .expect("the host must retain its stable edit gateway");
        let mut contains_created = false;
        edit_gateway
            .with_world(&mut |world| contains_created = world.contains_entity(created))
            .expect("the edit gateway must remain readable while play is attached");
        assert!(contains_created);

        let play_gateway = controller
            .gateway_for(WorldDomain::Play(play_instance))
            .expect("the host should route the attached play-domain gateway by instance");
        let mut play_read = |_world: &World| {};
        assert_eq!(
            play_gateway.with_world(&mut play_read),
            Err(GatewayError::RequiresSerializedAccess)
        );

        controller
            .play_sessions()
            .request_stop()
            .expect("the test play backend should stop before terminal detach");
        controller
            .detach_terminal_play_gateway()
            .expect("terminal detachment must leave the edit facade in place");
        assert!(controller
            .gateway_for(WorldDomain::Play(play_instance))
            .is_none());

        {
            let mut shell = controller.shell().lock();
            assert!(shell
                .state
                .apply_intent(EditorIntent::Undo)
                .expect("undo after play detaches should use the preserved edit history"));
            assert_eq!(
                shell.state.viewport_controller.selection().active_primary(),
                Some(selection_before)
            );
        }
        assert!(!authoring_level.with_world(|world| world.contains_entity(created)));
    }
}
