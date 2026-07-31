//! Bounded completion-byte reservation shared by raster workers and the frame drain.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

pub(super) struct CompletionByteBudget {
    max_bytes: usize,
    state: Mutex<CompletionByteBudgetState>,
    changed: Condvar,
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
            changed: Condvar::new(),
        }
    }

    pub(super) fn reserve_while<F>(&self, byte_count: usize, is_cancelled: F) -> bool
    where
        F: Fn() -> bool,
    {
        let mut state = self.lock_state();
        loop {
            if state.closed || is_cancelled() {
                return false;
            }
            let fits_budget = byte_count <= self.max_bytes
                && state.reserved_bytes.saturating_add(byte_count) <= self.max_bytes;
            // A single oversized bitmap must still make progress so its source-cache owner can
            // reject it explicitly rather than leaving a permanently pending work id.
            let single_oversized_result = state.reserved_bytes == 0 && byte_count > self.max_bytes;
            if fits_budget || single_oversized_result {
                state.reserved_bytes = state.reserved_bytes.saturating_add(byte_count);
                return true;
            }

            let (next_state, _) = self
                .changed
                .wait_timeout(state, Duration::from_millis(1))
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
        }
    }

    #[cfg(test)]
    pub(super) fn try_reserve(&self, byte_count: usize) -> bool {
        let mut state = self.lock_state();
        if state.closed
            || (byte_count <= self.max_bytes
                && state.reserved_bytes.saturating_add(byte_count) > self.max_bytes)
            || (byte_count > self.max_bytes && state.reserved_bytes != 0)
        {
            return false;
        }
        state.reserved_bytes = state.reserved_bytes.saturating_add(byte_count);
        true
    }

    pub(super) fn release(&self, byte_count: usize) {
        let mut state = self.lock_state();
        state.reserved_bytes = state.reserved_bytes.saturating_sub(byte_count);
        self.changed.notify_all();
    }

    pub(super) fn close(&self) {
        let mut state = self.lock_state();
        state.closed = true;
        self.changed.notify_all();
    }

    fn lock_state(&self) -> MutexGuard<'_, CompletionByteBudgetState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
