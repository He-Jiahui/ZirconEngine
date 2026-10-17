//! UI 驱动、事件管理器与停止清理钩子的 Core 注册入口。
//! 本模块组装服务描述；项目加载方与宿主继续负责资源工作及表面生命周期。

use std::sync::Arc;

use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::render::GRAPHICS_MODULE_NAME;
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::framework::ui::UI_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};
use crate::ui::event_ui::UiEventManager;

#[path = "module/lifecycle.rs"]
mod lifecycle;
#[path = "module/runtime_driver.rs"]
mod runtime_driver;

pub use runtime_driver::{UiConfig, UiRuntimeDriver, UI_CONFIG_KEY};

#[cfg(test)]
#[path = "module/tests/cases.rs"]
mod tests;

/// Core 解析和停止清理使用的 UI 驱动注册名，与描述中生成的限定名称对应。
pub const UI_RUNTIME_DRIVER_NAME: &str = "UiModule.Driver.UiRuntimeDriver";
/// Core 解析 UI 事件管理器使用的注册名；服务实例由描述中的工厂创建。
pub const UI_EVENT_MANAGER_NAME: &str = "UiModule.Manager.UiEventManager";

/// 交给应用或内建插件组注册的 UI 模块标记。
/// 它通过 [`EngineModule`] 返回模块信息与描述，自身不保存服务实例。
#[derive(Clone, Copy, Debug, Default)]
pub struct UiModule;

/// 为 Core 组装 UI 服务及停止清理钩子的模块描述。
///
/// # 返回与调用时机
/// 返回新描述；本次调用仅保存工厂，实例由 Core 在服务初始化时创建。
/// 驱动工厂读取该 Core 的配置，管理器工厂创建默认实例。
///
/// # 依赖与生命周期
/// 描述声明 Input、Scene、Graphics 模块依赖，以及事件管理器对 UI 驱动的依赖。
/// 停止钩子关闭既有驱动的准入；参见 [`UiRuntimeDriver`] 和 [`UiEventManager`]。
pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(UI_MODULE_NAME, "Runtime UI widgets and layout")
        .with_init_level(InitLevel::Scene)
        .with_lifecycle(Arc::new(lifecycle::UiModuleLifecycle))
        .with_module_dependency(ModuleDependencySpec::named(INPUT_MODULE_NAME))
        .with_module_dependency(ModuleDependencySpec::named(SCENE_MODULE_NAME))
        .with_module_dependency(ModuleDependencySpec::named(GRAPHICS_MODULE_NAME))
        .with_driver(DriverDescriptor::new(
            qualified_name(UI_MODULE_NAME, ServiceKind::Driver, "UiRuntimeDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let core = core
                    .upgrade()
                    .ok_or(crate::core::CoreError::RuntimeUnavailable)?;
                Ok(Arc::new(UiRuntimeDriver::from_core(&core)?) as ServiceObject)
            }),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(UI_MODULE_NAME, ServiceKind::Manager, "UiEventManager"),
            StartupMode::Immediate,
            vec![dependency_on(
                UI_MODULE_NAME,
                ServiceKind::Driver,
                "UiRuntimeDriver",
            )],
            factory(|_| Ok(Arc::new(UiEventManager::default()) as ServiceObject)),
        ))
}

impl EngineModule for UiModule {
    fn module_name(&self) -> &'static str {
        UI_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Runtime UI widgets and layout"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
