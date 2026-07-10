use std::sync::{Mutex, MutexGuard};

use crate::core::editor_operation::{EditorOperationRegistry, EditorOperationStack};

pub(crate) struct EditorOperationState {
    state: Mutex<EditorOperationStateData>,
}

pub(crate) struct EditorOperationStateData {
    pub(crate) registry: EditorOperationRegistry,
    pub(crate) stack: EditorOperationStack,
}

impl EditorOperationState {
    pub(crate) fn with_builtin_operations() -> Self {
        Self {
            state: Mutex::new(EditorOperationStateData {
                registry: EditorOperationRegistry::with_builtin_operations(),
                stack: EditorOperationStack::default(),
            }),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, EditorOperationStateData> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn stack(&self) -> EditorOperationStack {
        self.lock().stack.clone()
    }
}
