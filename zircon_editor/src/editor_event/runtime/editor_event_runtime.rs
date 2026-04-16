use std::sync::{Arc, Mutex};

use zircon_editor_ui::EditorUiControlService;

use crate::editor_event::{EditorEventJournal, EditorTransientUiState};
use crate::{EditorManager, EditorState};

use super::runtime_inner::EditorEventRuntimeInner;

pub struct EditorEventRuntime {
    pub(super) inner: Mutex<EditorEventRuntimeInner>,
}

impl EditorEventRuntime {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let runtime = Self {
            inner: Mutex::new(EditorEventRuntimeInner {
                state,
                manager,
                transient: EditorTransientUiState::default(),
                journal: EditorEventJournal::default(),
                control_service: EditorUiControlService::default(),
                next_event_id: 0,
                next_sequence: 0,
                revision: 0,
                dragging_gizmo: false,
            }),
        };
        runtime.refresh_reflection();
        runtime
    }
}
