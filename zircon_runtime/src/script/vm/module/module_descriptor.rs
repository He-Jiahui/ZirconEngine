use std::sync::Arc;

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor, StartupMode};
use crate::engine_module::{
    dependency_on, factory, plugin_context, plugin_factory, qualified_name,
};

use crate::script::{
    PluginHostDriver, VmPluginManager, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME,
    VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SCRIPT_MODULE_NAME, "VM plugin hosting and hot reload")
        .with_init_level(InitLevel::Post)
        .with_module_dependency(ModuleDependencySpec::named(SCENE_MODULE_NAME))
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
        .with_manager(crate::core::ManagerDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Manager,
                "VmPluginManager",
            ),
            StartupMode::Immediate,
            vec![dependency_on(
                SCRIPT_MODULE_NAME,
                crate::core::ServiceKind::Driver,
                "PluginHostDriver",
            )],
            factory(|core| {
                let core_handle = core.upgrade().ok_or_else(|| {
                    crate::core::CoreError::Initialization(
                        VM_PLUGIN_MANAGER_NAME.to_string(),
                        "manager context no longer has a live core handle".to_string(),
                    )
                })?;
                let host = core_handle
                    .resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)?
                    .clone();
                let manager = VmPluginManager::with_plugin_context_and_host_exports(
                    plugin_context(VM_PLUGIN_RUNTIME_NAME, core.clone()),
                    host.registry(),
                    host.host_exports(),
                );
                manager.install_reflection_world_extension(&core_handle)?;
                Ok(manager as ServiceObject)
            }),
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
                crate::core::ServiceKind::Manager,
                "VmPluginManager",
            )],
            plugin_factory(|context| {
                let manager = context
                    .core
                    .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)?;
                Ok(manager as ServiceObject)
            }),
        ))
}
