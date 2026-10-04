use crate::script::{VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage};

/// 后端在管理器选定家族后装载一个包；返回的实例由协调器独占管理，宿主上下文包含本次装载的能力和所有者。
pub trait VmBackend: Send + Sync {
    fn backend_name(&self) -> &str;

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError>;
}
