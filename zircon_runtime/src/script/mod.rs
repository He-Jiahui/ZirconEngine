//! VM plugin contracts, host handles, and hot reload coordination.

mod vm;

pub use vm::{
    builtin_host_capabilities, builtin_host_module_descriptors, discover_vm_plugin_package,
    discover_vm_plugin_packages, module_descriptor, register_builtin_host_modules,
    render_script_host_modules_markdown, write_script_host_modules_markdown,
    BuiltinVmBackendFamily, CapabilitySet, DiscoveredVmPluginPackage, HostCapabilityRecord,
    HostExportCallback, HostExportFunction, HostExportModuleRecord, HostExportRegistry, HostHandle,
    HostRegistry, HotReloadCoordinator, PluginHostDriver, PluginSlotId,
    ScriptHostInterfaceMarkdownOptions, ScriptModule, UnavailableVmBackend, VmBackend,
    VmBackendFamily, VmBackendRegistry, VmError, VmPluginHostContext, VmPluginInstance,
    VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle, VmPluginSlotRecord, VmStateBlob, ZrVmExecutionMode,
    ZrVmPluginProjectSource, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
    VM_PLUGIN_RUNTIME_NAME,
};
