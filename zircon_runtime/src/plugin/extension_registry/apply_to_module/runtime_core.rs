use crate::core::ModuleDescriptor;

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    /// 为宿主模块描述符附加插件管理器；它会冻结注册阶段，调用方随后将描述符交给核心运行时。
    pub fn apply_to_module(&mut self, mut descriptor: ModuleDescriptor) -> ModuleDescriptor {
        self.finalize();
        descriptor.managers.extend(self.managers().iter().cloned());
        descriptor
    }
}
