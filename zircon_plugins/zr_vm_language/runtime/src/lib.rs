mod backend;
mod call_site;
mod capability;
pub mod host_interface;
mod module;
mod plugin;
#[cfg(feature = "backend-zr-vm")]
mod real_backend;
mod reflection_host;

pub use backend::{ZrVmBackend, ZrVmBackendFamily};
pub use call_site::{CallSiteError, CompiledCallSite, ParamLayout, ScriptCallTable};
pub use capability::{
    RUNTIME_CAPABILITIES, ZR_VM_LANGUAGE_RUNTIME_CAPABILITY, ZR_VM_PROJECT_BACKEND_CAPABILITY,
};
pub use module::{
    module_descriptor, register_zr_vm_backend, ZrVmLanguageBackendRegistration,
    ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME, ZR_VM_LANGUAGE_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, vm_system_dispatcher_id,
    ZrVmLanguageRuntimePlugin, ZR_VM_GC_STEP_SYSTEM, ZR_VM_LANGUAGE_DIST_CRATE_NAME,
    ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY,
};
pub use reflection_host::{ReflectionHostError, ReflectionHostModule};
pub use zircon_runtime::script::{
    HostHandle, VmGcBudget, VmGcDiagnostics, VmGcRootRegistrationError, VmGcRootRegistry,
    VmGcRootToken, VmGcSlotStepReport, VmGcStepOutcome, VmGcStepReport, VmObjectId, VmObjectRef,
    VmObjectRefError, VmStateBlob, VmStateFieldRename, VmStateMigrationError, VmStateObject,
    VmStateSchema, VmStateTypeIdentity, VmStateTypeSchema, DEFAULT_VM_GC_MAX_MICROS_PER_FRAME,
    VM_GC_DIAGNOSTICS_HISTORY_CAPACITY, VM_STATE_SCHEMA_VERSION_V2,
};

pub const PLUGIN_ID: &str = "zr_vm_language";
pub const ZR_VM_PROJECT_BACKEND_SELECTOR: &str = "zr_vm:project";

#[cfg(test)]
mod tests;
