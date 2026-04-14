//! Module and plugin descriptor helpers built on top of the core runtime.

use std::fmt;
use std::sync::Arc;
use zircon_core::{CoreError, ServiceFactory, ServiceObject};

pub use zircon_core::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, LifecycleState,
    ManagerDescriptor, ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor,
    RegistryName, ServiceKind, StartupMode,
};

#[derive(Clone, Debug, Default)]
pub struct StubDriver {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct StubManager {
    pub name: String,
}

pub fn qualified_name(module: &str, kind: ServiceKind, service: &str) -> RegistryName {
    RegistryName::from_parts(module, kind, service)
}

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

pub trait EngineModule: Send + Sync + fmt::Debug {
    fn descriptor(&self) -> ModuleDescriptor;
}

pub fn dependency_on(module: &str, kind: ServiceKind, service: &str) -> DependencySpec {
    DependencySpec::named(qualified_name(module, kind, service))
}

pub fn factory(
    builder: impl Fn(&zircon_core::CoreHandle) -> Result<ServiceObject, CoreError>
        + Send
        + Sync
        + 'static,
) -> ServiceFactory {
    Arc::new(builder)
}

pub fn module_context(
    module_name: impl Into<String>,
    core: zircon_core::CoreWeak,
) -> ModuleContext {
    ModuleContext {
        module_name: module_name.into(),
        core,
    }
}

pub fn plugin_context(
    plugin_name: impl Into<String>,
    core: zircon_core::CoreWeak,
) -> PluginContext {
    PluginContext {
        plugin_name: plugin_name.into(),
        core,
    }
}
