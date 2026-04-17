use std::sync::Arc;

use zircon_core::{ModuleDescriptor, ServiceObject, StartupMode};
use zircon_module::{dependency_on, factory, qualified_name};

use crate::{PluginHostDriver, VmPluginManager, SCRIPT_MODULE_NAME};

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SCRIPT_MODULE_NAME, "VM plugin hosting and hot reload")
        .with_driver(zircon_core::DriverDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Driver,
                "PluginHostDriver",
            ),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(PluginHostDriver::default()) as ServiceObject)),
        ))
        .with_manager(zircon_core::ManagerDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Manager,
                "VmPluginManager",
            ),
            StartupMode::Immediate,
            vec![dependency_on(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Driver,
                "PluginHostDriver",
            )],
            factory(|_| Ok(Arc::new(VmPluginManager::unavailable()) as ServiceObject)),
        ))
}
