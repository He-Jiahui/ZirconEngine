use std::sync::Arc;

use super::session_slot::SessionSlot;

/// Keeps a session action visible to the destroy quiescence barrier.
pub(super) struct SessionActionGuard {
    slot: Arc<SessionSlot>,
}

impl SessionActionGuard {
    pub(super) fn new(slot: Arc<SessionSlot>) -> Self {
        Self { slot }
    }
}

impl Drop for SessionActionGuard {
    fn drop(&mut self) {
        self.slot.finish_action();
    }
}
