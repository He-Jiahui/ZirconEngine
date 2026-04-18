use std::sync::Arc;

use crate::ui::EditorUiControlService;

use crate::core::editor_event::{EditorEventJournal, EditorTransientUiState};
use crate::{EditorManager, EditorState};

pub(super) struct EditorEventRuntimeInner {
    pub(super) state: EditorState,
    pub(super) manager: Arc<EditorManager>,
    pub(super) transient: EditorTransientUiState,
    pub(super) journal: EditorEventJournal,
    pub(super) control_service: EditorUiControlService,
    pub(super) next_event_id: u64,
    pub(super) next_sequence: u64,
    pub(super) revision: u64,
    pub(super) dragging_gizmo: bool,
}
