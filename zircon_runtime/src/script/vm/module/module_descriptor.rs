use std::sync::Arc;

use crate::core::{ModuleDescriptor, ServiceObject, StartupMode};
use crate::engine_module::{dependency_on, factory, qualified_name};

use crate::script::{
    PluginHostDriver, VmPluginManager, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME,
    VM_PLUGIN_RUNTIME_NAME,
};

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SCRIPT_MODULE_NAME, "VM plugin hosting and hot reload")
        .with_driver(crate::core::DriverDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Driver,
                "PluginHostDriver",
            ),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(PluginHostDriver::default()) as ServiceObject)),
        ))
        .with_plugin(crate::core::PluginDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Plugin,
                "VmPluginRuntime",
            ),
            StartupMode::Immediate,
            vec![dependency_on(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Driver,
                "PluginHostDriver",
            )],
            factory(|core| {
                let host = core
                    .resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)?
                    .registry();
                Ok(Arc::new(VmPluginManager::with_builtin_backends(host)) as ServiceObject)
            }),
        ))
        .with_manager(crate::core::ManagerDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Manager,
                "VmPluginManager",
            ),
            StartupMode::Immediate,
            vec![dependency_on(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Plugin,
                "VmPluginRuntime",
            )],
            factory(|core| {
                let runtime = core.resolve_plugin::<VmPluginManager>(VM_PLUGIN_RUNTIME_NAME)?;
                Ok(runtime as ServiceObject)
            }),
        ))
}
