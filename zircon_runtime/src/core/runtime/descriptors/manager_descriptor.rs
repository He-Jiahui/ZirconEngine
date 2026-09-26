use std::fmt;
use std::sync::Arc;

use crate::core::StartupMode;

use super::{DependencySpec, RegistryName, ServiceFactory};

/// 管理器的注册声明，供模块图安排依赖、启动和卸载。
///
/// 工厂只在解析或即时启动时执行；返回的实例随后由服务注册表持有。
#[derive(Clone)]
pub struct ManagerDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Arc<[DependencySpec]>,
    pub factory: ServiceFactory,
}

impl ManagerDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies: dependencies.into(),
            factory,
        }
    }
}

impl fmt::Debug for ManagerDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagerDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}
