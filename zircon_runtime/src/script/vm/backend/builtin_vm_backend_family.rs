use std::sync::Arc;

use super::{MockVmBackend, UnavailableVmBackend, VmBackend, VmBackendFamily, VmError};

#[derive(Debug, Default)]
pub struct BuiltinVmBackendFamily;

impl VmBackendFamily for BuiltinVmBackendFamily {
    fn family_name(&self) -> &str {
        "builtin"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "builtin:mock" | "mock" => Ok(Arc::new(MockVmBackend)),
            "builtin:unavailable" | "unavailable" => Ok(Arc::new(UnavailableVmBackend)),
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn selectors(&self) -> Vec<String> {
        vec![
            "builtin:mock".to_string(),
            "mock".to_string(),
            "builtin:unavailable".to_string(),
            "unavailable".to_string(),
        ]
    }
}
