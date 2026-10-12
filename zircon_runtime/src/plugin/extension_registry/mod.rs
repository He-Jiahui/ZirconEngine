//! 插件注册中心把可撤销的贡献分流至元数据、系统、资源和桥接接口，再按目标运行时安装。
#[path = "access.rs"]
mod access;
#[path = "apply_to_asset_manager.rs"]
mod apply_to_asset_manager;
#[path = "apply_to_module.rs"]
mod apply_to_module;
#[path = "apply_to_project_manager.rs"]
mod apply_to_project_manager;
#[cfg(feature = "ui")]
#[path = "apply_to_ui.rs"]
mod apply_to_ui;
#[path = "apply_to_world.rs"]
mod apply_to_world;
#[path = "owner.rs"]
mod owner;
#[path = "ownership.rs"]
mod ownership;
#[path = "register.rs"]
mod register;
#[path = "runtime_extension_registry.rs"]
mod runtime_extension_registry;
#[path = "typed_extension_point.rs"]
mod typed_extension_point;
#[path = "validation.rs"]
mod validation;

pub use owner::PluginModuleId;
pub use ownership::ExtensionOwnership;
pub use runtime_extension_registry::RuntimeExtensionRegistry;
pub use typed_extension_point::{
    ExtensionKey, ExtensionSlot, FrozenExtensionTable, TypedExtensionPoint,
};
