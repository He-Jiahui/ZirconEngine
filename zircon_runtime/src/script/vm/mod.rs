mod backend;
mod capability_set;
mod gameplay_host;
mod handles;
mod host;
mod module;
mod plugin;
mod runtime;
mod runtime_context;
mod scene_hook;
mod tests;

pub use backend::{BuiltinVmBackendFamily, VmBackendFamily, ZrVmBackend, ZrVmBackendFamily};
pub use backend::{UnavailableVmBackend, VmBackend, VmBackendRegistry, VmError};
pub use capability_set::CapabilitySet;
pub use gameplay_host::register_gameplay_host_module;
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
    VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy,
    VmPluginInstance, VmPluginManagementPolicy, VmPluginManifest, VmPluginMemoryPolicy,
    VmPluginPackage, VmPluginPackageSource, VmStateBlob, ZrVmExecutionMode,
    ZrVmPluginProjectSource,
};
pub use runtime::{HotReloadCoordinator, VmPluginManager, VmPluginSlotRecord, VmPluginSlotState};
pub use runtime_context::{
    current_script_runtime_call_context, script_float, with_script_runtime_call_context,
    ScriptRuntimeCallContext,
};
pub use scene_hook::{
    script_scene_fixed_update_hook_registration, script_scene_update_hook_registration,
    ScriptSceneLifecyclePhase, ScriptSceneRuntimeHook,
};
