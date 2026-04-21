use std::sync::Arc;

use crate::core::editor_event::EditorEventJournal;
use crate::ui::control::EditorUiControlService;
use crate::ui::host::EditorManager;
use crate::ui::workbench::reflection::EditorTransientUiState;
use crate::ui::workbench::state::EditorState;

pub(crate) struct EditorEventRuntimeInner {
    pub(crate) state: EditorState,
    pub(crate) manager: Arc<EditorManager>,
    pub(crate) transient: EditorTransientUiState,
    pub(crate) journal: EditorEventJournal,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) next_event_id: u64,
    pub(crate) next_sequence: u64,
    pub(crate) revision: u64,
    pub(crate) dragging_gizmo: bool,
}

impl EditorEventRuntimeInner {
    pub(crate) fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        Self {
            state,
            manager,
            transient: EditorTransientUiState::default(),
            journal: EditorEventJournal::default(),
            control_service: EditorUiControlService::default(),
            next_event_id: 0,
            next_sequence: 0,
            revision: 0,
            dragging_gizmo: false,
        }
    }
}
