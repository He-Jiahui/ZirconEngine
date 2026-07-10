use std::sync::Arc;

use crate::core::context::EditorContext;
use crate::core::editing::operation_state::EditorOperationState;
use crate::core::play::{EditorPlayBridge, SharedEditorRuntimePlayModeBackend};
use crate::scene::viewport::GizmoDragState;
use crate::ui::workbench::shell_state::WorkbenchShellState;
use crate::ui::workbench::state::EditorState;

use super::EditorManager;

/// UI host coordinator over independently synchronized editor owners.
pub struct EditorHostEventController {
    context: Arc<EditorContext>,
    shell: Arc<WorkbenchShellState>,
    operations: Arc<EditorOperationState>,
    play_bridge: Arc<EditorPlayBridge>,
    gizmo_drag: Arc<GizmoDragState>,
}

impl EditorHostEventController {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let controller = Self {
            context: manager.context().clone(),
            shell: Arc::new(WorkbenchShellState::new(state, manager)),
            operations: Arc::new(EditorOperationState::with_builtin_operations()),
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

    pub(crate) fn operations(&self) -> &EditorOperationState {
        &self.operations
    }

    pub(crate) fn play_bridge(&self) -> &EditorPlayBridge {
        &self.play_bridge
    }

    pub(crate) fn gizmo_drag(&self) -> &GizmoDragState {
        &self.gizmo_drag
    }
}
