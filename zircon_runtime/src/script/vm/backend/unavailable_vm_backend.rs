use crate::script::{VmBackend, VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage};

#[derive(Debug, Default)]
pub struct UnavailableVmBackend;

impl VmBackend for UnavailableVmBackend {
    fn backend_name(&self) -> &str {
        "unavailable"
    }

    fn load_package(
        &self,
        _package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Err(VmError::BackendUnavailable(
            "zr_vm integration is not wired yet".to_string(),
        ))
    }
}
