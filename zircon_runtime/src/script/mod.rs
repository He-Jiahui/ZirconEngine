//! VM plugin contracts, host handles, and hot reload coordination.

mod vm;

pub use vm::{
    discover_vm_plugin_package, discover_vm_plugin_packages, module_descriptor,
    BuiltinVmBackendFamily, CapabilitySet, DiscoveredVmPluginPackage, HostHandle, HostRegistry,
    HotReloadCoordinator, PluginHostDriver, PluginSlotId, ScriptModule, UnavailableVmBackend,
    VmBackend, VmBackendFamily, VmBackendRegistry, VmError, VmPluginHostContext, VmPluginInstance,
    VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle, VmPluginSlotRecord, VmStateBlob, PLUGIN_HOST_DRIVER_NAME,
    SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
