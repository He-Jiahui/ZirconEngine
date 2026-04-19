use std::sync::Arc;

use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, PluginDescriptor, ServiceKind,
    ServiceObject, StartupMode,
};

use super::names::qualified_name;
use super::stub_driver::StubDriver;
use super::stub_manager::StubManager;

pub fn stub_driver_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> DriverDescriptor {
    let name = qualified_name(module, ServiceKind::Driver, service);
    let service_name = name.to_string();
    DriverDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| {
            Ok(Arc::new(StubDriver {
                name: service_name.clone(),
            }) as ServiceObject)
        }),
    )
}

pub fn stub_manager_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> ManagerDescriptor {
    let name = qualified_name(module, ServiceKind::Manager, service);
    let service_name = name.to_string();
    ManagerDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| {
            Ok(Arc::new(StubManager {
                name: service_name.clone(),
            }) as ServiceObject)
        }),
    )
}

pub fn stub_plugin_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> PluginDescriptor {
    let name = qualified_name(module, ServiceKind::Plugin, service);
    let service_name = name.to_string();
    PluginDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| {
            Ok(Arc::new(StubManager {
                name: service_name.clone(),
            }) as ServiceObject)
        }),
    )
}

pub fn stub_module_descriptor(
    module: &str,
    description: &str,
    driver_service: &str,
    manager_service: &str,
) -> ModuleDescriptor {
    ModuleDescriptor::new(module, description)
        .with_driver(stub_driver_descriptor(
            module,
            driver_service,
            StartupMode::Immediate,
        ))
        .with_manager(stub_manager_descriptor(
            module,
            manager_service,
            StartupMode::Lazy,
        ))
}
