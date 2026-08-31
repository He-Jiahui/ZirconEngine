mod backend;
mod behavior_bridge;
mod capability_set;
mod gameplay_host;
mod gc_bridge;
mod handles;
mod host;
mod host_interface;
mod module;
mod plugin;
mod reflection;
mod runtime;
mod runtime_context;
mod scene_system;
mod tests;

pub use backend::{BuiltinVmBackendFamily, VmBackendFamily};
pub use backend::{UnavailableVmBackend, VmBackend, VmBackendRegistry, VmError};
pub use behavior_bridge::VmScriptBehaviorBridge;
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
    VmPluginHostContext, VmPluginSlotLifecycle, VmReflectionSchemaInstaller,
    BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME,
    VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
pub use host_interface::{
    VmBehaviorNodeRegistration, VmCallbackHandle, VmEditorOperationRegistration,
    VmHostInterfaceError, VmHostInterfaceRegistry, VmInterfaceCaller, VmRpcHandlerRegistration,
    VmSystemRegistration, VmSystemStage, VM_BT_NODE_CAPABILITY, VM_EDITOR_OPERATION_CAPABILITY,
    VM_HOST_INTERFACE_MODULE, VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};
pub use module::{module_descriptor, ScriptModule};
pub use plugin::{
    discover_vm_plugin_package, discover_vm_plugin_package_with_limits,
    discover_vm_plugin_packages, discover_vm_plugin_packages_with_limits, migrate_vm_state_blob,
    DiscoveredVmPluginPackage, VmPluginDiscoveryLimits, VmPluginDiscoveryRequest,
    VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy,
    VmPluginInstance, VmPluginManagementPolicy, VmPluginManagementPolicyError,
    VmPluginManagementPolicyResult, VmPluginManifest, VmPluginMemoryPolicy, VmPluginPackage,
    VmPluginPackageSource, VmStateBlob, VmStateFieldValue, VmStateMigrationError, VmStateObject,
    VmStateSchema, VmStateTypeIdentity, VmStateTypeSchema, ZrVmExecutionMode,
    ZrVmPluginProjectSource, VM_STATE_SCHEMA_VERSION_V3,
};
pub use reflection::{
    VmReflectionCatalog, VmReflectionError, VmReflectionRegistrySnapshot, VmReflectionSchema,
    VM_REFLECTION_WORLD_EXTENSION_NAME,
};
pub use runtime::{HotReloadCoordinator, VmPluginManager, VmPluginSlotRecord, VmPluginSlotState};
pub(crate) use runtime_context::runtime_context_for_frame;
pub use runtime_context::{script_float, VmReflectionWorldAccess, VmReflectionWorldOperation};
pub(crate) use runtime_context::{with_script_runtime_call_context, ScriptRuntimeCallContext};
#[cfg(feature = "test-support")]
pub use runtime_context::{with_script_runtime_test_context, ScriptRuntimeTestContext};
pub use scene_system::{
    ScriptSceneLifecyclePhase, ScriptSceneRuntimeSystem, SCRIPT_SCENE_FIXED_UPDATE_SYSTEM,
    SCRIPT_SCENE_RUNTIME_SYSTEM_SET, SCRIPT_SCENE_UPDATE_SYSTEM,
};
