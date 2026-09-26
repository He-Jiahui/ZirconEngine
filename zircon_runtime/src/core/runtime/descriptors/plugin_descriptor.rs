use std::fmt;
use std::sync::Arc;

use crate::core::StartupMode;

use super::{DependencySpec, PluginFactory, RegistryName};

/// 插件服务的注册声明；与普通服务共用冻结依赖图和生命周期门禁。
///
/// 插件工厂额外接收 PluginContext，包路径可以缺席，不能假定总有文件来源。
#[derive(Clone)]
pub struct PluginDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Arc<[DependencySpec]>,
    pub factory: PluginFactory,
}

impl PluginDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: PluginFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies: dependencies.into(),
            factory,
        }
    }
}

impl fmt::Debug for PluginDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}
