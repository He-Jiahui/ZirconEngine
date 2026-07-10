use std::sync::{Mutex, MutexGuard};

/// Physics manager APIs stay available after a worker panics while holding shared state.
pub(super) fn recover_lock<T>(state: &Mutex<T>) -> MutexGuard<'_, T> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
