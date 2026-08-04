use std::sync::{Arc, Mutex, MutexGuard};

use super::viewport_state::ViewportState;

#[derive(Clone)]
pub(crate) struct RetainedViewportController {
    pub(super) shared: Arc<Mutex<ViewportState>>,
    pub(super) viewport_lifecycle: Arc<Mutex<()>>,
}

impl RetainedViewportController {
    pub(super) fn lock_shared(&self) -> MutexGuard<'_, ViewportState> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_viewport_lifecycle(&self) -> MutexGuard<'_, ()> {
        self.viewport_lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
