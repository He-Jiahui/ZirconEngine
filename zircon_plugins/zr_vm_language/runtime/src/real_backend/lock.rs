use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Proof of the process-wide VM lock. Only this module can construct it.
/// The token cannot be shared across threads to authorize concurrent VM calls.
pub(super) struct ZrVmLockGuard {
    _guard: MutexGuard<'static, ()>,
    _not_sync: PhantomData<Cell<()>>,
}

pub(super) fn acquire_zr_vm_lock() -> ZrVmLockGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    ZrVmLockGuard {
        _guard: guard,
        _not_sync: PhantomData,
    }
}

#[cfg(test)]
#[path = "tests/lock.rs"]
mod tests;
