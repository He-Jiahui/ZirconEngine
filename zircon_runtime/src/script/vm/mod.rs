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
    builtin_host_capabilities, builtin_host_module_descriptors, register_builtin_host_modules,
    render_script_host_modules_markdown, write_script_host_modules_markdown, HostCapabilityRecord,
    HostExportCallback, HostExportFunction, HostExportModuleRecord, HostExportRegistry,
    HostRegistry, PluginHostDriver, ScriptHostInterfaceMarkdownOptions, VmPluginHostContext,
    VmPluginSlotLifecycle, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
    VM_PLUGIN_RUNTIME_NAME,
};
pub use module::{module_descriptor, ScriptModule};
pub use plugin::{
    discover_vm_plugin_package, discover_vm_plugin_packages, DiscoveredVmPluginPackage,
    VmPluginInstance, VmPluginManifest, VmPluginPackage, VmPluginPackageSource, VmStateBlob,
    ZrVmExecutionMode, ZrVmPluginProjectSource,
};
pub use runtime::{HotReloadCoordinator, VmPluginManager, VmPluginSlotRecord};
