use crate::script::{HostRegistry, VmBackend, VmError, VmPluginInstance, VmPluginPackage};

#[derive(Debug, Default)]
pub struct UnavailableVmBackend;

impl VmBackend for UnavailableVmBackend {
    fn backend_name(&self) -> &str {
        "unavailable"
    }

    fn load_package(
        &self,
        _package: &VmPluginPackage,
        _host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Err(VmError::BackendUnavailable(
            "zr_vm integration is not wired yet".to_string(),
        ))
    }
}
