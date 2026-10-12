#[cfg(feature = "native-host")]
#[path = "ids/module_id.rs"]
mod module_id;
#[path = "ids/plugin_id.rs"]
mod plugin_id;

#[cfg(feature = "native-host")]
pub use module_id::BuiltinRuntimeModuleId;
pub use plugin_id::RuntimePluginId;
