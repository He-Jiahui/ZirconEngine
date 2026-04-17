mod capability_set;
mod handles;
mod backend;
mod host;
mod module;
mod plugin;
mod runtime;
mod tests;

pub use capability_set::CapabilitySet;
pub use backend::{UnavailableVmBackend, VmBackend, VmError};
pub use handles::{HostHandle, PluginSlotId};
pub use host::{
    HostRegistry, PluginHostDriver, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME,
    VM_PLUGIN_MANAGER_NAME,
};
pub use module::{module_descriptor, ScriptModule};
pub use plugin::{VmPluginInstance, VmPluginManifest, VmPluginPackage, VmStateBlob};
pub use runtime::{HotReloadCoordinator, VmPluginManager};
