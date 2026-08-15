//! Bounded completion-byte reservation shared by raster workers and the frame drain.

use std::sync::{Mutex, MutexGuard};

pub(super) struct CompletionByteBudget {
    max_bytes: usize,
    state: Mutex<CompletionByteBudgetState>,
}

#[derive(Default)]
struct CompletionByteBudgetState {
    reserved_bytes: usize,
    closed: bool,
}

impl CompletionByteBudget {
    pub(super) fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            state: Mutex::new(CompletionByteBudgetState::default()),
        }
    }

    pub(super) fn try_reserve(&self, byte_count: usize) -> bool {
        let mut state = self.lock_state();
        if state.closed
            || byte_count > self.max_bytes
            || state.reserved_bytes.saturating_add(byte_count) > self.max_bytes
        {
            return false;
        }
        state.reserved_bytes = state.reserved_bytes.saturating_add(byte_count);
        true
    }

    pub(super) fn release(&self, byte_count: usize) {
        let mut state = self.lock_state();
        state.reserved_bytes = state.reserved_bytes.saturating_sub(byte_count);
    }

    pub(super) fn close(&self) {
        let mut state = self.lock_state();
        state.closed = true;
    }

    fn lock_state(&self) -> MutexGuard<'_, CompletionByteBudgetState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
