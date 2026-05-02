use std::sync::{Mutex, MutexGuard};

use super::editor_event_runtime_inner::EditorEventRuntimeInner;

pub struct EditorEventRuntime {
    pub(crate) inner: Mutex<super::editor_event_runtime_inner::EditorEventRuntimeInner>,
}

impl EditorEventRuntime {
    pub(crate) fn lock_inner(&self) -> MutexGuard<'_, EditorEventRuntimeInner> {
        // Editor callbacks should not permanently brick readback/query paths after a recovered panic.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
