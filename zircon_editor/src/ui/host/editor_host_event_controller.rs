use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::core::commands::{EditorCommandRegistryHandle, EditorKeymap};
use crate::core::context::EditorContext;
use crate::core::gateway::{EditorRuntimeGateway, SharedEditorRuntimeGateway};
use crate::core::play::{EditorPlayBridge, SharedEditorRuntimePlayModeBackend};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerRegistry,
};
use crate::scene::viewport::GizmoDragState;
use crate::ui::workbench::shell_state::WorkbenchShellState;
use crate::ui::workbench::state::EditorState;

use super::EditorManager;

const FIRST_PLAY_SESSION_GENERATION: u64 = 1;

/// UI host coordinator over independently synchronized editor owners.
pub struct EditorHostEventController {
    context: Arc<EditorContext>,
    shell: Arc<WorkbenchShellState>,
    commands: EditorCommandRegistryHandle,
    keymap: EditorKeymap,
    play_bridge: Arc<EditorPlayBridge>,
    gizmo_drag: Arc<GizmoDragState>,
    runtime_event_consumers: EditorRuntimeEventConsumerHost,
    next_play_session_generation: AtomicU64,
}

impl EditorHostEventController {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let context = manager.context().clone();
        let commands = context.commands().clone();
        let controller = Self {
            context: context.clone(),
            shell: Arc::new(WorkbenchShellState::new(state, manager)),
            commands,
            keymap: EditorKeymap::default_workbench(),
            play_bridge: Arc::new(EditorPlayBridge::new()),
            gizmo_drag: Arc::new(GizmoDragState::default()),
            runtime_event_consumers: EditorRuntimeEventConsumerHost::new(context.gateway().clone()),
            next_play_session_generation: AtomicU64::new(FIRST_PLAY_SESSION_GENERATION),
        };
        controller.refresh_reflection();
        controller
    }

    pub fn context(&self) -> &Arc<EditorContext> {
        &self.context
    }

    pub fn set_runtime_play_mode_backend(&self, backend: SharedEditorRuntimePlayModeBackend) {
        self.play_bridge.set_backend(backend);
    }

    pub fn set_runtime_gateway(&self, gateway: SharedEditorRuntimeGateway) {
        self.context.gateway().replace(gateway);
    }

    pub fn register_runtime_event_consumers(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        self.runtime_event_consumers.register(registry)
    }

    pub fn begin_runtime_event_consumers(&self) -> Result<(), EditorRuntimeEventConsumerError> {
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

    pub fn pump_runtime_event_consumers(&self) -> Result<usize, EditorRuntimeEventConsumerError> {
        if self
            .runtime_event_consumers
            .active_play_session_id()
            .is_none()
        {
            return Ok(0);
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
        let advanced = self.context.gateway().tick_frame().map_err(|message| {
            EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "runtime.frame.tick".to_string(),
                message: message.to_string(),
            }
        })?;
        if !advanced {
            return Err(EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "runtime.frame.tick".to_string(),
                message: "runtime did not expose frame ticking".to_string(),
            });
        }
        self.runtime_event_consumers.pump()
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

    pub(crate) fn play_bridge(&self) -> &EditorPlayBridge {
        &self.play_bridge
    }

    pub(crate) fn gizmo_drag(&self) -> &GizmoDragState {
        &self.gizmo_drag
    }
}
