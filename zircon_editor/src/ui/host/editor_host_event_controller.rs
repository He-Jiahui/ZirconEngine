use std::sync::Arc;

use crate::core::commands::EditorCommandRegistryHandle;
use crate::core::context::EditorContext;
use crate::core::play::{EditorPlayBridge, SharedEditorRuntimePlayModeBackend};
use crate::scene::viewport::GizmoDragState;
use crate::ui::workbench::shell_state::WorkbenchShellState;
use crate::ui::workbench::state::EditorState;

use super::EditorManager;

/// UI host coordinator over independently synchronized editor owners.
pub struct EditorHostEventController {
    context: Arc<EditorContext>,
    shell: Arc<WorkbenchShellState>,
    commands: EditorCommandRegistryHandle,
    play_bridge: Arc<EditorPlayBridge>,
    gizmo_drag: Arc<GizmoDragState>,
}

impl EditorHostEventController {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let context = manager.context().clone();
        let commands = context.commands().clone();
        let controller = Self {
            context,
            shell: Arc::new(WorkbenchShellState::new(state, manager)),
            commands,
            play_bridge: Arc::new(EditorPlayBridge::new()),
            gizmo_drag: Arc::new(GizmoDragState::default()),
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

    pub(crate) fn shell(&self) -> &WorkbenchShellState {
        &self.shell
    }

    pub(crate) fn commands(&self) -> &EditorCommandRegistryHandle {
        &self.commands
    }

    pub(crate) fn play_bridge(&self) -> &EditorPlayBridge {
        &self.play_bridge
    }

    pub(crate) fn gizmo_drag(&self) -> &GizmoDragState {
        &self.gizmo_drag
    }
}
