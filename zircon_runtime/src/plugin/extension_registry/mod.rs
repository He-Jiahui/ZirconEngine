mod access;
mod apply_to_asset_manager;
mod apply_to_module;
mod apply_to_ui;
mod apply_to_world;
mod owner;
mod ownership;
mod register;
mod runtime_extension_registry;
mod typed_extension_point;
mod validation;

pub use owner::PluginModuleId;
pub use ownership::ExtensionOwnership;
pub use runtime_extension_registry::RuntimeExtensionRegistry;
pub use typed_extension_point::{ExtensionKey, ExtensionSlot, FrozenExtensionTable};
