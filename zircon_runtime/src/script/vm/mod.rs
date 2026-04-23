mod backend;
mod capability_set;
mod handles;
mod host;
mod module;
mod plugin;
mod runtime;
mod tests;

pub use backend::{BuiltinVmBackendFamily, VmBackendFamily};
pub use backend::{UnavailableVmBackend, VmBackend, VmBackendRegistry, VmError};
pub use capability_set::CapabilitySet;
pub use handles::{HostHandle, PluginSlotId};
pub use host::{
    HostRegistry, PluginHostDriver, VmPluginHostContext, VmPluginSlotLifecycle,
    PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
pub use module::{module_descriptor, ScriptModule};
pub use plugin::{
    discover_vm_plugin_package, discover_vm_plugin_packages, DiscoveredVmPluginPackage,
    VmPluginInstance, VmPluginManifest, VmPluginPackage, VmPluginPackageSource, VmStateBlob,
};
pub use runtime::{HotReloadCoordinator, VmPluginManager, VmPluginSlotRecord};
