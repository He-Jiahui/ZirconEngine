use crate::script::{VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage};

pub trait VmBackend: Send + Sync {
    fn backend_name(&self) -> &str;

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError>;
}
