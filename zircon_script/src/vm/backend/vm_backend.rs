use crate::{HostRegistry, VmError, VmPluginInstance, VmPluginPackage};

pub trait VmBackend: Send + Sync {
    fn backend_name(&self) -> &str;

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError>;
}
