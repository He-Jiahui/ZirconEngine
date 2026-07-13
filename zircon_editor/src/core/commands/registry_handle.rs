use std::sync::{Arc, Mutex, MutexGuard};

use super::EditorCommandRegistry;

/// Shared owner used by every runtime command discovery and invocation surface.
#[derive(Clone, Debug)]
pub struct EditorCommandRegistryHandle {
    registry: Arc<Mutex<EditorCommandRegistry>>,
}

impl EditorCommandRegistryHandle {
    pub fn new(registry: EditorCommandRegistry) -> Self {
        Self {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    pub fn default_workbench() -> Self {
        Self::new(EditorCommandRegistry::default_workbench())
    }

    pub fn lock(&self) -> MutexGuard<'_, EditorCommandRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
