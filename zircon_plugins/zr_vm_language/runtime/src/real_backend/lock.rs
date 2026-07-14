use std::sync::{Mutex, MutexGuard, OnceLock};

pub(super) fn acquire_zr_vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zr_vm_real_backend_runtime_lock_recovers_after_poison() {
        let poison_result = std::thread::spawn(|| {
            let _guard = acquire_zr_vm_lock();
            panic!("poison zr_vm real backend runtime lock");
        })
        .join();

        assert!(poison_result.is_err());

        let _guard = acquire_zr_vm_lock();
    }
}
