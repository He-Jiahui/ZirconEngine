use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::{VmBackend, VmBackendFamily, VmError};

#[derive(Default)]
pub struct VmBackendRegistry {
    families: Mutex<BTreeMap<String, Arc<dyn VmBackendFamily>>>,
}

impl fmt::Debug for VmBackendRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmBackendRegistry")
            .field("families", &self.names())
            .finish()
    }
}

impl VmBackendRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_family(&self, family: Arc<dyn VmBackendFamily>) -> String {
        let name = family.family_name().to_string();
        self.families.lock().unwrap().insert(name.clone(), family);
        name
    }

    pub fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        let families = self
            .families
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();

        if let Some((family_name, _)) = selector.split_once(':') {
            if let Some(family) = families
                .iter()
                .find(|family| family.family_name() == family_name)
            {
                return family.resolve(selector);
            }
        }

        for family in families {
            if let Ok(backend) = family.resolve(selector) {
                return Ok(backend);
            }
        }

        Err(VmError::UnknownBackend(selector.to_string()))
    }

    pub fn contains(&self, selector: &str) -> bool {
        self.resolve(selector).is_ok()
    }

    pub fn names(&self) -> Vec<String> {
        let mut selectors = self
            .families
            .lock()
            .unwrap()
            .values()
            .flat_map(|family| family.selectors())
            .collect::<Vec<_>>();
        selectors.sort();
        selectors.dedup();
        selectors
    }
}
