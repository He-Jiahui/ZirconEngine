use std::fmt;
use std::sync::Arc;

use super::{DriverDescriptor, ManagerDescriptor, ModuleDependencySpec, PluginDescriptor};
use crate::core::runtime::lifecycle::{InitLevel, ModuleLifecycle, NoopModuleLifecycle};

/// 一个模块在注册阶段提交的生命周期、依赖和服务声明。
///
/// CoreHandle 首次激活前冻结这些声明并验证图；之后的状态属于注册表条目，
/// 调用方不应把修改已提交描述符的本地副本视为运行时重配置。
#[derive(Clone)]
pub struct ModuleDescriptor {
    pub name: String,
    pub description: String,
    pub init_level: InitLevel,
    pub module_dependencies: Vec<ModuleDependencySpec>,
    pub lifecycle: Arc<dyn ModuleLifecycle>,
    pub drivers: Vec<DriverDescriptor>,
    pub managers: Vec<ManagerDescriptor>,
    pub plugins: Vec<PluginDescriptor>,
}

impl ModuleDescriptor {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            lifecycle: Arc::new(NoopModuleLifecycle),
            drivers: Vec::new(),
            managers: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.init_level = init_level;
        self
    }

    /// 声明先启动的模块；跨模块服务引用也必须有对应的模块边。
    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module_dependencies.push(dependency);
        self
    }

    pub fn with_lifecycle(mut self, lifecycle: Arc<dyn ModuleLifecycle>) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    pub fn with_driver(mut self, descriptor: DriverDescriptor) -> Self {
        self.drivers.push(descriptor);
        self
    }

    pub fn with_manager(mut self, descriptor: ManagerDescriptor) -> Self {
        self.managers.push(descriptor);
        self
    }

    pub fn with_plugin(mut self, descriptor: PluginDescriptor) -> Self {
        self.plugins.push(descriptor);
        self
    }
}

impl fmt::Debug for ModuleDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleDescriptor")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("init_level", &self.init_level)
            .field("module_dependencies", &self.module_dependencies)
            .field("lifecycle", &"ModuleLifecycle")
            .field("drivers", &self.drivers)
            .field("managers", &self.managers)
            .field("plugins", &self.plugins)
            .finish()
    }
}
