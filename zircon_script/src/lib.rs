//! VM plugin contracts, host handles, and hot reload coordination.

mod vm;

pub use vm::{
    module_descriptor, HostRegistry, HotReloadCoordinator, PluginHostDriver, UnavailableVmBackend,
    VmBackend, VmError, VmPluginInstance, VmPluginManager, VmPluginManifest, VmPluginPackage,
    VmStateBlob, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
};
