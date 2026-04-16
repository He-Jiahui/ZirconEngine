use super::{
    host_registry::HostRegistry, vm_error::VmError, vm_plugin_instance::VmPluginInstance,
    vm_plugin_package::VmPluginPackage,
};

pub trait VmBackend: Send + Sync {
    fn backend_name(&self) -> &str;

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError>;
}
