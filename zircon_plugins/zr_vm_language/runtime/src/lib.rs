use std::sync::Arc;

use zircon_runtime::script::VmPluginManager;

mod backend;
mod module;
#[cfg(feature = "real-zr-vm")]
mod real_backend;

pub use backend::{ZrVmBackend, ZrVmBackendFamily};
pub use module::{
    module_descriptor, ZrVmLanguageBackendRegistration, ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
    ZR_VM_LANGUAGE_MODULE_NAME,
};

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

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn register_zr_vm_backend(manager: &VmPluginManager) -> String {
    manager.register_family(Arc::new(ZrVmBackendFamily))
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "ZrVM Language",
        zircon_runtime::RuntimePluginId::ZrVmLanguage,
        "zircon_plugin_zr_vm_language_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_enabled_by_default(false)
    .with_capability("runtime.plugin.zr_vm_language")
    .with_capability("runtime.script.backend.zr_vm_project")
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
mod tests {
    use super::*;

    #[test]
    fn zr_vm_language_registration_reports_backend_capability() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == ZR_VM_LANGUAGE_MODULE_NAME));
        assert!(report
            .package_manifest
            .capabilities
            .contains(&"runtime.script.backend.zr_vm_project".to_string()));
    }

    #[test]
    fn zr_vm_backend_family_resolves_project_selector() {
        let manager = zircon_runtime::script::VmPluginManager::mock();
        register_zr_vm_backend(&manager);

        assert!(manager
            .backend_names()
            .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
        manager
            .select_default_backend(ZR_VM_PROJECT_BACKEND_SELECTOR)
            .unwrap();
        assert_eq!(
            manager.selected_backend_name(),
            ZR_VM_PROJECT_BACKEND_SELECTOR
        );
    }

    #[test]
    fn zr_vm_runtime_module_registers_backend_with_vm_manager() {
        let runtime = zircon_runtime::core::CoreRuntime::new();
        runtime
            .register_module(zircon_runtime::script::module_descriptor())
            .unwrap();
        runtime.register_module(module_descriptor()).unwrap();
        runtime
            .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
            .unwrap();
        runtime.activate_module(ZR_VM_LANGUAGE_MODULE_NAME).unwrap();

        let registration = runtime
            .handle()
            .resolve_plugin::<ZrVmLanguageBackendRegistration>(
                ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
            )
            .unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<zircon_runtime::script::VmPluginManager>(
                zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
            )
            .unwrap();

        assert_eq!(registration.selector, "zr_vm");
        assert!(manager
            .backend_names()
            .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
    }
}
