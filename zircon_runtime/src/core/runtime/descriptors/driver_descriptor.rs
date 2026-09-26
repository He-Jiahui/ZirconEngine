use std::fmt;
use std::sync::Arc;

use crate::core::StartupMode;

use super::{DependencySpec, RegistryName, ServiceFactory};

/// 驱动的注册声明；工厂会在依赖可解析后按启动策略调用。
///
/// 构造描述符不创建实例，生命周期和卸载顺序由所属模块的冻结图决定。
#[derive(Clone)]
pub struct DriverDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Arc<[DependencySpec]>,
    pub factory: ServiceFactory,
}

impl DriverDescriptor {
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

impl fmt::Debug for DriverDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}
