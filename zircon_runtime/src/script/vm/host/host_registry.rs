use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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
    pub fn register_capability(&self, label: impl Into<String>) -> HostHandle {
        let handle = HostHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        self.handles.lock().unwrap().insert(
            handle,
            HostCapabilityRecord {
                handle,
                label: label.into(),
            },
        );
        handle
    }

    pub fn capability(&self, handle: HostHandle) -> Option<HostCapabilityRecord> {
        self.handles.lock().unwrap().get(&handle).cloned()
    }

    pub fn capabilities(&self) -> Vec<HostCapabilityRecord> {
        let mut records = self
            .handles
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.handle.get());
        records
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        self.handles.lock().unwrap().contains_key(&handle)
    }
}
