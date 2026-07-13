mod backend;
mod capability_set;
mod gameplay_host;
mod gc_bridge;
mod handles;
mod host;
mod host_interface;
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
pub use gc_bridge::{
    HostHandle, VmGcBudget, VmGcDiagnostics, VmGcRootRegistrationError, VmGcRootRegistry,
    VmGcRootToken, VmGcSlotStepReport, VmGcStepOutcome, VmGcStepReport, VmObjectId, VmObjectRef,
    VmObjectRefError, DEFAULT_VM_GC_MAX_MICROS_PER_FRAME, VM_GC_DIAGNOSTICS_HISTORY_CAPACITY,
};
pub use handles::PluginSlotId;
pub use host::{
    builtin_host_capabilities, builtin_host_module_descriptors, register_bridge_host_module,
    register_builtin_host_modules, render_script_host_modules_markdown,
    write_script_host_modules_markdown, HostCapabilityRecord, HostExportCallback,
    HostExportFunction, HostExportModuleRecord, HostExportRegistry, HostRegistry,
    HostRegistryError, PluginHostDriver, ScriptBridgeCall, ScriptBridgeMethodDescriptor,
    ScriptCallSite, ScriptCallSiteId, ScriptCallTable, ScriptHostInterfaceMarkdownOptions,
    VmPluginHostContext, VmPluginSlotLifecycle, BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE,
    PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
pub use host_interface::{
    VmBehaviorNodeRegistration, VmCallbackHandle, VmEditorOperationRegistration,
    VmHostInterfaceError, VmHostInterfaceRegistry, VmInterfaceCaller, VmRpcHandlerRegistration,
    VmSystemRegistration, VmSystemStage, VM_BT_NODE_CAPABILITY, VM_EDITOR_OPERATION_CAPABILITY,
    VM_HOST_INTERFACE_MODULE, VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};
pub use module::{module_descriptor, ScriptModule};
pub use plugin::{
    discover_vm_plugin_package, discover_vm_plugin_packages, migrate_vm_state_blob,
    DiscoveredVmPluginPackage, VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy,
    VmPluginHotReloadPolicy, VmPluginInstance, VmPluginManagementPolicy,
    VmPluginManagementPolicyError, VmPluginManagementPolicyResult, VmPluginManifest,
    VmPluginMemoryPolicy, VmPluginPackage, VmPluginPackageSource, VmStateBlob, VmStateFieldRename,
    VmStateMigrationError, VmStateObject, VmStateSchema, VmStateTypeIdentity, VmStateTypeSchema,
    ZrVmExecutionMode, ZrVmPluginProjectSource, VM_STATE_SCHEMA_VERSION_V2,
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
