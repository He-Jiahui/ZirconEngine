use std::sync::{Arc, Mutex, MutexGuard};

use super::viewport_state::ViewportState;

#[derive(Clone)]
pub(crate) struct RetainedViewportController {
    pub(super) shared: Arc<Mutex<ViewportState>>,
}

impl RetainedViewportController {
    pub(super) fn lock_shared(&self) -> MutexGuard<'_, ViewportState> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
