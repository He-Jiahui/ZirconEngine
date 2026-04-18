use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::{VmBackend, VmError};

#[derive(Default)]
pub struct VmBackendRegistry {
    backends: Mutex<BTreeMap<String, Arc<dyn VmBackend>>>,
}

impl fmt::Debug for VmBackendRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmBackendRegistry")
            .field("backends", &self.names())
            .finish()
    }
}

impl VmBackendRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, backend: Arc<dyn VmBackend>) -> String {
        let name = backend.backend_name().to_string();
        self.register_named(name.clone(), backend);
        name
    }

    pub fn register_named(&self, name: impl Into<String>, backend: Arc<dyn VmBackend>) {
        self.backends.lock().unwrap().insert(name.into(), backend);
    }

    pub fn resolve(&self, name: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        self.backends
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| VmError::UnknownBackend(name.to_string()))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.backends.lock().unwrap().contains_key(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.backends.lock().unwrap().keys().cloned().collect()
    }
}
