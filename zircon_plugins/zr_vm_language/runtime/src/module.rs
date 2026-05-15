use std::sync::Arc;

use zircon_runtime::core::{ModuleDescriptor, ServiceObject, StartupMode};
use zircon_runtime::engine_module::{dependency_on, plugin_factory, qualified_name};
use zircon_runtime::script::{SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME};

use crate::register_zr_vm_backend;

pub const ZR_VM_LANGUAGE_MODULE_NAME: &str = "ZrVmLanguageModule";
pub const ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME: &str =
    "ZrVmLanguageModule.Plugin.ZrVmBackendRegistration";

#[derive(Debug)]
pub struct ZrVmLanguageBackendRegistration {
    pub selector: String,
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
