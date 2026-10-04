use crate::core::{ManagerDescriptor, ModuleDescriptor};

use super::super::RuntimeExtensionRegistry;

// 模块与管理器切片供目录合并和宿主装配读取；完成注册前仍可被新的贡献改变。
impl RuntimeExtensionRegistry {
    pub fn managers(&self) -> &[ManagerDescriptor] {
        self.managers.values()
    }

    pub fn modules(&self) -> &[ModuleDescriptor] {
        self.modules.values()
    }
}
