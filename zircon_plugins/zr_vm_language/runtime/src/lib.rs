use zircon_runtime::script::VmPluginManager;

mod module;

pub use module::{
    module_descriptor, ZrVmLanguageBackendRegistration, ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
    ZR_VM_LANGUAGE_MODULE_NAME,
};
pub use zircon_runtime::script::{ZrVmBackend, ZrVmBackendFamily};

pub const PLUGIN_ID: &str = "zr_vm_language";
pub const ZR_VM_PROJECT_BACKEND_SELECTOR: &str = "zr_vm:project";

#[derive(Clone, Debug)]
pub struct ZrVmLanguageRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl ZrVmLanguageRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ZrVmLanguageRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for ZrVmLanguageRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_scene_hook(
            zircon_runtime::script::script_scene_fixed_update_hook_registration(),
        )?;
        registry
            .register_scene_hook(zircon_runtime::script::script_scene_update_hook_registration())
    }
}

pub fn register_zr_vm_backend(manager: &VmPluginManager) -> String {
    manager.register_family(std::sync::Arc::new(ZrVmBackendFamily))
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "ZrVM Language",
        zircon_runtime::builtin::RuntimePluginId::ZrVmLanguage,
        "zircon_plugin_zr_vm_language_runtime",
    )
    .with_category("runtime")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_enabled_by_default(false)
    .with_capability("runtime.plugin.zr_vm_language")
    .with_capability("runtime.script.backend.zr_vm_project")
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.plugin.zr_vm_language",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.script.backend.zr_vm_project",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
}

pub fn runtime_plugin() -> ZrVmLanguageRuntimePlugin {
    ZrVmLanguageRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        "runtime.plugin.zr_vm_language",
        "runtime.script.backend.zr_vm_project",
    ]
}

#[cfg(test)]
mod tests;
