//! VM plugin contracts, host handles, and hot reload coordination.

mod vm;

pub use vm::{
    builtin_host_capabilities, builtin_host_module_descriptors,
    current_script_runtime_call_context, discover_vm_plugin_package, discover_vm_plugin_packages,
    module_descriptor, register_builtin_host_modules, register_gameplay_host_module,
    render_script_host_modules_markdown, script_float, script_scene_fixed_update_hook_registration,
    script_scene_update_hook_registration, with_script_runtime_call_context,
    write_script_host_modules_markdown, BuiltinVmBackendFamily, CapabilitySet,
    DiscoveredVmPluginPackage, HostCapabilityRecord, HostExportCallback, HostExportFunction,
    HostExportModuleRecord, HostExportRegistry, HostHandle, HostRegistry, HotReloadCoordinator,
    PluginHostDriver, PluginSlotId, ScriptHostInterfaceMarkdownOptions, ScriptModule,
    ScriptRuntimeCallContext, ScriptSceneLifecyclePhase, ScriptSceneRuntimeHook,
    UnavailableVmBackend, VmBackend, VmBackendFamily, VmBackendRegistry, VmError,
    VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHostContext,
    VmPluginHotReloadPolicy, VmPluginInstance, VmPluginManagementPolicy, VmPluginManager,
    VmPluginManifest, VmPluginMemoryPolicy, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle, VmPluginSlotRecord, VmPluginSlotState, VmStateBlob, ZrVmBackend,
    ZrVmBackendFamily, ZrVmExecutionMode, ZrVmPluginProjectSource, PLUGIN_HOST_DRIVER_NAME,
    SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
