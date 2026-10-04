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

#[path = "play_gizmo.rs"]
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
        controller.ensure_live_scene_viewport_sessions();
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
#[path = "tests/editor_host_event_controller_lifecycle_contract_tests.rs"]
mod lifecycle_contract_tests;
