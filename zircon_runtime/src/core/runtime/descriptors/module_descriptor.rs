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
    /// 模块身份键；提交注册时验证名称并拒绝重名，服务名称也必须归属于它。
    pub name: String,
    /// 供描述和诊断使用的文本；不参与依赖排序。
    pub description: String,
    /// 初始化层级；模块不得依赖更晚层级的提供方。
    pub init_level: InitLevel,
    /// 显式模块依赖，既安排先启动的模块，也授权跨模块服务依赖；传递可达不能替代直接声明。
    pub module_dependencies: Vec<ModuleDependencySpec>,
    /// 注册表保留的共享生命周期回调；build、ready、finish 与受限 cleanup 的职责见对应 trait。
    pub lifecycle: Arc<dyn ModuleLifecycle>,
    /// 本模块拥有的驱动声明；注册时准备服务槽位，尚不创建实例。
    pub drivers: Vec<DriverDescriptor>,
    /// 本模块拥有的管理器声明，随后与其他服务共同进入冻结依赖图。
    pub managers: Vec<ManagerDescriptor>,
    /// 本模块拥有的插件服务声明；工厂上下文由实际解析入口提供。
    pub plugins: Vec<PluginDescriptor>,
}

impl ModuleDescriptor {
    /// 用 `name`、`description` 创建本地声明，默认 Post 层级、空依赖/服务与空操作生命周期。
    /// 返回值尚未注册，名称验证、图校验和工厂执行分别留在后续入口。
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

    /// 消费并返回修改后的声明；仅设置层级，依赖层级关系在冻结时核对。
    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.init_level = init_level;
        self
    }

    /// 声明先启动的模块；跨模块服务引用也必须有对应的模块边。
    /// 消费并返回追加后的本地声明；此处不去重或确认提供方存在。
    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module_dependencies.push(dependency);
        self
    }

    /// 消费并返回持有新共享回调的声明；设置时不调用任何生命周期方法。
    pub fn with_lifecycle(mut self, lifecycle: Arc<dyn ModuleLifecycle>) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// 取得驱动描述符并追加到返回的本地声明；提交注册前不创建服务槽位或实例。
    pub fn with_driver(mut self, descriptor: DriverDescriptor) -> Self {
        self.drivers.push(descriptor);
        self
    }

    /// 取得管理器描述符并追加到返回的本地声明；校验与实例化由后续入口负责。
    pub fn with_manager(mut self, descriptor: ManagerDescriptor) -> Self {
        self.managers.push(descriptor);
        self
    }

    /// 取得插件描述符并追加到返回的本地声明；不会在此调用插件工厂。
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
