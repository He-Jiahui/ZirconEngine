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
