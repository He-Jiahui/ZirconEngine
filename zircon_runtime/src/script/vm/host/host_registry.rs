use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::super::HostHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCapabilityRecord {
    pub handle: HostHandle,
    pub label: String,
}

#[derive(Clone, Debug, Default)]
pub struct HostRegistry {
    next_handle: Arc<AtomicU64>,
    handles: Arc<Mutex<HashMap<HostHandle, HostCapabilityRecord>>>,
}

impl HostRegistry {
    fn lock_handles(&self) -> MutexGuard<'_, HashMap<HostHandle, HostCapabilityRecord>> {
        self.handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn register_capability(&self, label: impl Into<String>) -> HostHandle {
        let handle = HostHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        self.lock_handles().insert(
            handle,
            HostCapabilityRecord {
                handle,
                label: label.into(),
            },
        );
        handle
    }

    pub fn capability(&self, handle: HostHandle) -> Option<HostCapabilityRecord> {
        self.lock_handles().get(&handle).cloned()
    }

    pub fn capabilities(&self) -> Vec<HostCapabilityRecord> {
        let mut records = self.lock_handles().values().cloned().collect::<Vec<_>>();
        records.sort_by_key(|record| record.handle.get());
        records
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        self.lock_handles().contains_key(&handle)
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    #[test]
    fn host_registry_accessors_recover_poisoned_handle_lock() {
        let registry = HostRegistry::default();

        let poison_result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = registry.handles.lock().unwrap();
            panic!("poison host handle registry");
        }));
        assert!(poison_result.is_err());

        let handle = registry.register_capability("test.capability");
        assert!(registry.is_valid(handle));
        assert_eq!(
            registry.capability(handle).unwrap(),
            HostCapabilityRecord {
                handle,
                label: "test.capability".to_string(),
            }
        );
        assert_eq!(registry.capabilities().len(), 1);
    }
}
