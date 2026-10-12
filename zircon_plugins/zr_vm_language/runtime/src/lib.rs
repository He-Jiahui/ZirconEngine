#[path = "backend.rs"]
mod backend;
#[path = "call_site/mod.rs"]
mod call_site;
#[path = "capability.rs"]
mod capability;
#[path = "host_interface/mod.rs"]
pub mod host_interface;
#[path = "module.rs"]
mod module;
#[path = "plugin.rs"]
mod plugin;
#[cfg(feature = "backend-zr-vm")]
#[path = "real_backend.rs"]
mod real_backend;
#[path = "reflection_host/mod.rs"]
mod reflection_host;

pub use backend::{ZrVmBackend, ZrVmBackendFamily};
pub use call_site::{CallSiteError, CompiledCallSite, ParamLayout, ScriptCallTable};
pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES,
    ZR_VM_LANGUAGE_DECLARATION, ZR_VM_LANGUAGE_RUNTIME_CAPABILITY,
    ZR_VM_PROJECT_BACKEND_CAPABILITY,
};
pub use module::{
    module_descriptor, register_zr_vm_backend, ZrVmLanguageBackendRegistration,
    ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME, ZR_VM_LANGUAGE_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, vm_system_dispatcher_id,
    ZrVmLanguageRuntimePlugin, ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM, ZR_VM_GC_STEP_SYSTEM,
    ZR_VM_LANGUAGE_DIST_CRATE_NAME, ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY,
    ZR_VM_LANGUAGE_MAIN_SYSTEM_SET,
};
#[cfg(feature = "backend-zr-vm")]
pub use real_backend::build_zr_vm_native_host_modules;
pub use reflection_host::{ReflectionHostError, ReflectionHostModule};
pub use zircon_runtime::script::{
    HostHandle, VmGcBudget, VmGcDiagnostics, VmGcRootRegistrationError, VmGcRootRegistry,
    VmGcRootToken, VmGcSlotStepReport, VmGcStepOutcome, VmGcStepReport, VmObjectId, VmObjectRef,
    VmObjectRefError, VmStateBlob, VmStateFieldValue, VmStateMigrationError, VmStateObject,
    VmStateSchema, VmStateTypeIdentity, VmStateTypeSchema, DEFAULT_VM_GC_MAX_MICROS_PER_FRAME,
    VM_GC_DIAGNOSTICS_HISTORY_CAPACITY, VM_STATE_SCHEMA_VERSION_V3,
};

pub const ZR_VM_PROJECT_BACKEND_SELECTOR: &str = "zr_vm:project";

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
