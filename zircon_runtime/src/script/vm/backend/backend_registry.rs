use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

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

    fn lock_families(&self) -> MutexGuard<'_, BTreeMap<String, Arc<dyn VmBackendFamily>>> {
        self.families
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn register_family(&self, family: Arc<dyn VmBackendFamily>) -> String {
        let name = family.family_name().to_string();
        self.lock_families().insert(name.clone(), family);
        name
    }

    pub fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        if let Some((family_name, _)) = selector.split_once(':') {
            let family = self.lock_families().get(family_name).cloned();
            if let Some(family) = family {
                return family.resolve(selector);
            }
        }

        let families = self.lock_families().values().cloned().collect::<Vec<_>>();
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
        let families = self.lock_families();
        let mut selectors = Vec::new();
        for family in families.values() {
            family.visit_selectors(&mut |selector| selectors.push(selector.to_owned()));
        }
        selectors.sort();
        selectors.dedup();
        selectors
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::script::{VmPluginHostContext, VmPluginInstance, VmPluginPackage};

    use super::*;

    struct TestBackend;

    impl VmBackend for TestBackend {
        fn backend_name(&self) -> &str {
            "test:backend"
        }

        fn load_package(
            &self,
            _package: &VmPluginPackage,
            _host: &VmPluginHostContext,
        ) -> Result<Box<dyn VmPluginInstance>, VmError> {
            Err(VmError::Operation(
                "test backend does not load packages".to_string(),
            ))
        }
    }

    struct TestBackendFamily;

    impl VmBackendFamily for TestBackendFamily {
        fn family_name(&self) -> &str {
            "test"
        }

        fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
            if selector == "test:backend" {
                Ok(Arc::new(TestBackend))
            } else {
                Err(VmError::UnknownBackend(selector.to_string()))
            }
        }

        fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
            visitor("test:backend");
        }
    }

    #[test]
    fn vm_backend_registry_accessors_recover_poisoned_family_lock() {
        let registry = VmBackendRegistry::new();

        let poison_result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = registry.families.lock().unwrap();
            panic!("poison backend family registry");
        }));
        assert!(poison_result.is_err());

        assert_eq!(
            registry.register_family(Arc::new(TestBackendFamily)),
            "test"
        );
        assert_eq!(registry.names(), vec!["test:backend".to_string()]);
        assert_eq!(
            registry.resolve("test:backend").unwrap().backend_name(),
            "test:backend"
        );
        assert!(registry.contains("test:backend"));
    }
}

#[cfg(test)]
#[path = "backend_registry/qualified_lookup_tests.rs"]
mod qualified_lookup_tests;
