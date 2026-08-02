use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::commands::{EditorCommandPaletteMru, EditorCommandRegistryHandle, EditorKeymap};
use crate::core::context::EditorContext;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::gateway::{
    EditorRuntimeFrameDemand, EditorRuntimeGatewayHandle, SharedEditorRuntimeGateway,
};
use crate::core::play::{
    PlayDomainLinkError, PlayInstanceId, PlayModeKind, PlaySessionController, PlayTransitionCause,
    SharedPlayBackend, SharedPluginBridgeActivation, WorldDomain,
};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerRegistry,
};
use crate::ui::workbench::shell_state::WorkbenchShellState;
use crate::ui::workbench::state::EditorState;

use super::play_pending_decision::PlayPendingEditDecisionAdapter;
use super::scene_inspection_publication::SceneInspectionPublication;
use super::EditorManager;

const FIRST_PLAY_SESSION_GENERATION: u64 = 1;

/// UI host coordinator over independently synchronized editor owners.
pub struct EditorHostEventController {
    context: Arc<EditorContext>,
    shell: Arc<WorkbenchShellState>,
    commands: EditorCommandRegistryHandle,
    keymap: EditorKeymap,
    play_sessions: Arc<PlaySessionController>,
    play_pending_decisions: PlayPendingEditDecisionAdapter,
    pub(super) scene_inspection_publication: Mutex<SceneInspectionPublication>,
    pub(super) runtime_event_consumers: EditorRuntimeEventConsumerHost,
    pub(super) plugin_registration_gate: Mutex<()>,
    next_play_session_generation: AtomicU64,
}

impl EditorHostEventController {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let context = manager.context().clone();
        let commands = context.commands().clone();
        let play_sessions = Arc::new(PlaySessionController::with_message_bus(
            context.bus().clone(),
        ));
        let controller = Self {
            context: context.clone(),
            shell: Arc::new(WorkbenchShellState::new(state, manager)),
            commands,
            keymap: manager.keymap().clone(),
            play_sessions: play_sessions.clone(),
            play_pending_decisions: PlayPendingEditDecisionAdapter::default(),
            scene_inspection_publication: Mutex::new(SceneInspectionPublication::default()),
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

    pub fn attach_play_gateway(
        &self,
        gateway: SharedEditorRuntimeGateway,
    ) -> Result<PlayInstanceId, PlayDomainLinkError> {
        self.play_sessions.attach_play_gateway(gateway)
    }

    pub fn detach_play_gateway(&self, instance: PlayInstanceId) -> Result<(), PlayDomainLinkError> {
        self.play_sessions.detach_play_gateway(instance)
    }

    pub fn gateway_for(&self, domain: WorldDomain) -> Option<EditorRuntimeGatewayHandle> {
        match domain {
            WorldDomain::Edit => Some(self.context.gateway().clone()),
            WorldDomain::Play(instance) => self.play_sessions.play_gateway(instance),
        }
    }

    pub fn register_runtime_event_consumers(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.runtime_event_consumers.register(registry)
    }

    pub fn begin_runtime_event_consumers(&self) -> Result<(), EditorRuntimeEventConsumerError> {
        if self.play_sessions.attached_world_domain().is_none() {
            return Err(EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "play.domain".to_string(),
                message: "no play gateway is attached".to_string(),
            });
        }
        let enabled_capabilities = self
            .shell
            .lock()
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let play_session_id = self
            .next_play_session_generation
            .fetch_add(1, Ordering::Relaxed);
        self.runtime_event_consumers
            .begin_play_session(play_session_id, &enabled_capabilities)
    }

    pub fn pump_runtime_event_consumers(
        &self,
    ) -> Result<EditorRuntimeFrameDemand, EditorRuntimeEventConsumerError> {
        let backend_transition = self.play_sessions.poll_backend().map_err(|error| {
            EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "play.backend.poll".to_string(),
                message: error.to_string(),
            }
        })?;
        if backend_transition.changed && backend_transition.mode == PlayModeKind::Edit {
            if self.runtime_event_consumer_session_active() {
                self.end_runtime_event_consumers()?;
            }
            self.publish_pending_edit_decision(backend_transition.pending_edit_prompt.as_ref())
                .map_err(|message| EditorRuntimeEventConsumerError::Gateway {
                    consumer_id: "play.pending-edit-decision".to_string(),
                    message,
                })?;
            {
                let mut shell = self.shell.lock();
                let _ = shell.state.exit_play_mode();
                match backend_transition.cause {
                    PlayTransitionCause::Crashed { exit_code } => shell.state.set_status_line(
                        format!("Runtime preview exited unexpectedly (code {exit_code:?})"),
                    ),
                    _ => shell.state.set_status_line("Runtime preview stopped"),
                }
            }
            self.refresh_reflection();
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        }
        if self
            .runtime_event_consumers
            .active_play_session_id()
            .is_none()
        {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        }
        let enabled_capabilities = self
            .shell
            .lock()
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        self.runtime_event_consumers
            .reconcile_enabled_capabilities(&enabled_capabilities)?;
        let Some(WorldDomain::Play(instance)) = self.play_sessions.attached_world_domain() else {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        };
        let Some(play_gateway) = self.play_sessions.play_gateway(instance) else {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        };
        let frame_demand = play_gateway.tick_frame().map_err(|message| {
            EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "runtime.frame.tick".to_string(),
                message: message.to_string(),
            }
        })?;
        self.runtime_event_consumers.pump()?;
        Ok(frame_demand)
    }

    pub fn end_runtime_event_consumers(&self) -> Result<(), EditorRuntimeEventConsumerError> {
        let play_session_id = self
            .runtime_event_consumers
            .active_play_session_id()
            .ok_or(EditorRuntimeEventConsumerError::NoActiveSession)?;
        self.runtime_event_consumers
            .end_play_session(play_session_id)
    }

    pub fn runtime_event_consumer_session_active(&self) -> bool {
        self.runtime_event_consumers
            .active_play_session_id()
            .is_some()
    }

    pub(crate) fn shell(&self) -> &WorkbenchShellState {
        &self.shell
    }

    pub(crate) fn commands(&self) -> &EditorCommandRegistryHandle {
        &self.commands
    }

    pub(crate) fn keymap(&self) -> &EditorKeymap {
        &self.keymap
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
}
