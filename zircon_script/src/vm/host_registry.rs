use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use zircon_manager::HostHandle;

#[derive(Clone, Debug, Default)]
pub struct HostRegistry {
    next_handle: Arc<AtomicU64>,
    handles: Arc<Mutex<HashMap<HostHandle, String>>>,
}

impl HostRegistry {
    pub fn register_capability(&self, label: impl Into<String>) -> HostHandle {
        let handle = HostHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        self.handles.lock().unwrap().insert(handle, label.into());
        handle
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        self.handles.lock().unwrap().contains_key(&handle)
    }
}
