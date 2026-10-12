#[cfg(feature = "native-host")]
mod module_id;
mod plugin_id;

#[cfg(feature = "native-host")]
pub use module_id::BuiltinRuntimeModuleId;
pub use plugin_id::RuntimePluginId;
