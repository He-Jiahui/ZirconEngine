use zircon_runtime::script::VmPluginManager;

mod capability;
mod module;
mod plugin;

pub use capability::{
    RUNTIME_CAPABILITIES, ZR_VM_LANGUAGE_RUNTIME_CAPABILITY, ZR_VM_PROJECT_BACKEND_CAPABILITY,
};
pub use module::{
    module_descriptor, ZrVmLanguageBackendRegistration, ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
    ZR_VM_LANGUAGE_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, ZrVmLanguageRuntimePlugin,
    ZR_VM_LANGUAGE_DIST_CRATE_NAME, ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY,
};
pub use zircon_runtime::script::{ZrVmBackend, ZrVmBackendFamily};

pub const PLUGIN_ID: &str = "zr_vm_language";
pub const ZR_VM_PROJECT_BACKEND_SELECTOR: &str = "zr_vm:project";

pub fn register_zr_vm_backend(manager: &VmPluginManager) -> String {
    manager.register_family(std::sync::Arc::new(ZrVmBackendFamily))
}

#[cfg(test)]
mod tests;
