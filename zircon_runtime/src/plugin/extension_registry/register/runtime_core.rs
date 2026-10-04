use crate::core::{ManagerDescriptor, ModuleDescriptor};
use crate::plugin::RuntimeExtensionRegistryError;

use super::super::validation::{validate_manager_plugin_id, validate_module_descriptor};
use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    /// 将管理器挂到包级 owner，供目录合并和模块装配；同名管理器在全局目录中唯一。
    pub fn register_manager(
        &mut self,
        plugin_id: impl Into<String>,
        descriptor: ManagerDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let plugin_id = plugin_id.into();
        validate_manager_plugin_id(&plugin_id)?;
        let manager_name = descriptor.name.to_string();
        if self.managers.contains_key(&manager_name) {
            return Err(RuntimeExtensionRegistryError::DuplicateManager(
                descriptor.name.to_string(),
            ));
        }
        let owner = self.intern_runtime_owner(&plugin_id)?;
        self.managers
            .register(owner, manager_name, descriptor)
            .expect("manager duplicate was prechecked");
        Ok(())
    }

    // BUG: [CR-PLUGIN-BOUNDARY-0101] 描述符名已是 `client.runtime` 时这里再附加 `.runtime`，模块登记为 `client.runtime.runtime` owner；对 `client.runtime` 撤销会留下模块；证据：contributions/extension.rs 的目标模块样例及本函数调用链。
    /// 目录注册模块贡献时使用；模块名既用于目标筛选，也必须与其它贡献的 owner 命名一致。
    pub fn register_module(
        &mut self,
        descriptor: ModuleDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_module_descriptor(&descriptor)?;
        if self.modules.contains_key(&descriptor.name) {
            return Err(RuntimeExtensionRegistryError::DuplicateModule(
                descriptor.name,
            ));
        }
        let owner = self.intern_runtime_owner(&descriptor.name)?;
        self.modules
            .register(owner, descriptor.name.clone(), descriptor)
            .expect("module duplicate was prechecked");
        Ok(())
    }
}
