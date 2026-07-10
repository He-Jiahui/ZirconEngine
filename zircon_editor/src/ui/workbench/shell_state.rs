use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_extension::EditorExtensionRegistration;
use crate::ui::control::EditorUiControlService;
use crate::ui::host::EditorManager;
use crate::ui::workbench::reflection::EditorTransientUiState;
use crate::ui::workbench::state::EditorState;

/// UI-only workbench authority removed from the headless editor core.
pub(crate) struct WorkbenchShellState {
    state: Mutex<WorkbenchShellStateData>,
}

pub(crate) struct WorkbenchShellStateData {
    pub(crate) state: EditorState,
    pub(crate) manager: Arc<EditorManager>,
    pub(crate) transient: EditorTransientUiState,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) editor_extensions: Vec<EditorExtensionRegistration>,
}

impl WorkbenchShellState {
    pub(crate) fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        Self {
            state: Mutex::new(WorkbenchShellStateData {
                state,
                manager,
                transient: EditorTransientUiState::default(),
                control_service: EditorUiControlService::default(),
                editor_extensions: Vec::new(),
            }),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, WorkbenchShellStateData> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
