use std::sync::{Mutex, MutexGuard};

use super::editor_event_runtime_state::EditorEventRuntimeState;

pub struct EditorEventRuntime {
    pub(crate) inner: Mutex<EditorEventRuntimeState>,
}

impl EditorEventRuntime {
    pub(crate) fn lock_inner(&self) -> MutexGuard<'_, EditorEventRuntimeState> {
        // Editor callbacks should not permanently brick readback/query paths after a recovered panic.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
