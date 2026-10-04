//! 管理器实现由各模块工厂登记到 CoreRuntime；本模块汇集框架 trait 的服务键、注册包装与查询入口。
//! 查询入口保存的是 Core 弱引用和版本化身份，实际服务在指定运行时中重新解析。
//! Stable engine-facing manager handles, service names, and resolver helpers.

mod resolver;
mod service;
mod service_names;

#[cfg(feature = "ai-contracts")]
pub use resolver::ai_manager_handle;
#[cfg(feature = "net-contracts")]
pub use resolver::net_manager_handle;
#[cfg(feature = "physics-contracts")]
pub use resolver::physics_manager_handle;
#[cfg(feature = "sound-contracts")]
pub use resolver::sound_manager_handle;
pub use resolver::{
    animation_manager_handle, config_manager_handle, input_action_manager_handle,
    input_manager_handle, level_manager_handle, navigation_manager_handle,
    platform_preference_storage_handle, render_framework_handle, rendering_manager_handle,
    resource_manager_handle, ManagerResolver,
};
pub use service::{
    manager_service_handle, resolve_manager_service, ManagerServiceHandle, ManagerServiceResolver,
    RegisteredManagerService,
};
#[cfg(feature = "ai-contracts")]
pub use service_names::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
pub use service_names::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
pub use service_names::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
pub use service_names::SOUND_MANAGER_NAME;
pub use service_names::{
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, INPUT_ACTION_MANAGER_NAME, INPUT_MANAGER_NAME,
    LEVEL_MANAGER_NAME, NAVIGATION_MANAGER_NAME, PLATFORM_MANAGER_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
