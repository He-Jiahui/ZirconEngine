use std::sync::Arc;

use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{ModuleDescriptor, StartupMode};
use zircon_runtime::engine_module::{dependency_on, plugin_factory, qualified_name};
use zircon_runtime::script::{
    VmPluginManager, ZrVmBackendFamily, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
};

pub const ZR_VM_LANGUAGE_MODULE_NAME: &str = "zr_vm_language.runtime";
pub const ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME: &str =
    "zr_vm_language.runtime.Plugin.ZrVmBackendRegistration";

#[derive(Debug)]
pub struct ZrVmLanguageBackendRegistration {
    pub selector: String,
}

pub fn register_zr_vm_backend(manager: &VmPluginManager) -> String {
    manager.register_family(Arc::new(ZrVmBackendFamily))
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        ZR_VM_LANGUAGE_MODULE_NAME,
        "ZrVM language backend registration",
    )
    .with_plugin(zircon_runtime::core::PluginDescriptor::new(
        qualified_name(
            ZR_VM_LANGUAGE_MODULE_NAME,
            zircon_runtime::core::ServiceKind::Plugin,
            "ZrVmBackendRegistration",
        ),
        StartupMode::Immediate,
        vec![dependency_on(
            SCRIPT_MODULE_NAME,
            zircon_runtime::core::ServiceKind::Manager,
            "VmPluginManager",
        )],
        plugin_factory(|context| {
            let core = context.core.upgrade().ok_or_else(|| {
                zircon_runtime::core::CoreError::Initialization(
                    ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME.to_string(),
                    "plugin context no longer has a live core handle".to_string(),
                )
            })?;
            let manager = core.resolve_manager::<zircon_runtime::script::VmPluginManager>(
                VM_PLUGIN_MANAGER_NAME,
            )?;
            Ok(Arc::new(ZrVmLanguageBackendRegistration {
                selector: register_zr_vm_backend(&manager),
            }) as ServiceObject)
        }),
    ))
}
