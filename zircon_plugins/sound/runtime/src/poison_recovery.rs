use std::sync::{Mutex, MutexGuard};

/// Recovers the last valid state when a worker panic poisons a sound mutex.
pub(crate) fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
