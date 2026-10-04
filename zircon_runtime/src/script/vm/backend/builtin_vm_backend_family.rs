//! 内置家族把诊断用 mock 与明确不可用的后端接入统一选择器协议；真实语言后端由外部家族注册，不在此构造。
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
#[path = "tests/builtin_vm_backend_family.rs"]
mod tests;
