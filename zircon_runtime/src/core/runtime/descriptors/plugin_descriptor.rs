use std::fmt;
use std::sync::Arc;

use crate::core::StartupMode;

use super::{DependencySpec, PluginFactory, RegistryName};

/// 插件服务的注册声明；与普通服务共用冻结依赖图和生命周期门禁。
///
/// 插件工厂额外接收 PluginContext，包路径可以缺席，不能假定总有文件来源。
#[derive(Clone)]
pub struct PluginDescriptor {
    /// 规范服务键；提交注册时须属于所属模块且种类为 Plugin。
    pub name: RegistryName,
    /// Immediate 参加启动计划；Lazy 通常按需解析，也可能作为其他服务的依赖被提前解析。
    pub startup_mode: StartupMode,
    /// 显式依赖声明的共享切片；克隆描述符共享此只读声明，图冻结后另取确定的遍历顺序。
    pub dependencies: Arc<[DependencySpec]>,
    /// 延迟工厂的共享所有者；本描述符及其构造器都不调用工厂。
    pub factory: PluginFactory,
}

impl PluginDescriptor {
    /// 按同名字段接收注册名、启动策略、依赖列表与工厂，返回尚未提交的声明。
    /// 依赖列表转为共享切片；这里不查询注册表，也不核对名称归属或执行生命周期。
    /// 提交与失败边界见 [`crate::core::CoreHandle::register_module`]；工厂合同见相应类型别名。
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
