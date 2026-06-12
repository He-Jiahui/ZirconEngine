use std::sync::{Mutex, MutexGuard, OnceLock};

pub(super) fn acquire_zr_vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("zr_vm runtime lock should not be poisoned")
}
