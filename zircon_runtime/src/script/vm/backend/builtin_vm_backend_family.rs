use std::sync::{Arc, LazyLock};

use super::{MockVmBackend, UnavailableVmBackend, VmBackend, VmBackendFamily, VmError};

static MOCK_BACKEND: LazyLock<Arc<dyn VmBackend>> = LazyLock::new(|| Arc::new(MockVmBackend));
static UNAVAILABLE_BACKEND: LazyLock<Arc<dyn VmBackend>> =
    LazyLock::new(|| Arc::new(UnavailableVmBackend));

#[derive(Debug, Default)]
pub struct BuiltinVmBackendFamily;

impl VmBackendFamily for BuiltinVmBackendFamily {
    fn family_name(&self) -> &str {
        "builtin"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "builtin:mock" | "mock" => Ok(Arc::clone(&MOCK_BACKEND)),
            "builtin:unavailable" | "unavailable" => Ok(Arc::clone(&UNAVAILABLE_BACKEND)),
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor("builtin:mock");
        visitor("mock");
        visitor("builtin:unavailable");
        visitor("unavailable");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{BuiltinVmBackendFamily, VmBackendFamily};

    #[test]
    fn builtin_backend_resolutions_share_arc_storage() {
        let family = BuiltinVmBackendFamily;
        let mock = family.resolve("builtin:mock").unwrap();
        let mock_alias = family.resolve("mock").unwrap();
        let unavailable = family.resolve("builtin:unavailable").unwrap();
        let unavailable_alias = family.resolve("unavailable").unwrap();

        assert!(Arc::ptr_eq(&mock, &mock_alias));
        assert!(Arc::ptr_eq(&unavailable, &unavailable_alias));
        assert!(!Arc::ptr_eq(&mock, &unavailable));
    }
}
