use super::{
    host_registry::HostRegistry, vm_backend::VmBackend, vm_error::VmError,
    vm_plugin_instance::VmPluginInstance, vm_plugin_package::VmPluginPackage,
};

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
