use std::sync::{Arc, Mutex};

use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;
use crate::core::editor_event::EditorEventRuntime;
use crate::ui::host::EditorManager;
use crate::ui::workbench::state::EditorState;

impl EditorEventRuntime {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let runtime = Self {
            inner: Mutex::new(EditorEventRuntimeInner::new(state, manager)),
        };
        runtime.refresh_reflection();
        runtime
    }
}
