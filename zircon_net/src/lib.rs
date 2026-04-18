//! Networking module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const NET_MODULE_NAME: &str = "NetModule";
pub const NET_DRIVER_NAME: &str = "NetModule.Driver.NetDriver";
pub const NET_MANAGER_NAME: &str = "NetModule.Manager.NetManager";

#[derive(Clone, Debug, Default)]
pub struct NetConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NetModule;

#[derive(Clone, Debug, Default)]
pub struct NetDriver;

#[derive(Clone, Debug, Default)]
pub struct NetManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(NET_MODULE_NAME, "Networking, RPC, and multiplayer sessions")
        .with_driver(DriverDescriptor::new(
            qualified_name(NET_MODULE_NAME, ServiceKind::Driver, "NetDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(NetDriver::default()) as _)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(NET_MODULE_NAME, ServiceKind::Manager, "NetManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                NET_MODULE_NAME,
                ServiceKind::Driver,
                "NetDriver",
            )],
            factory(|_| Ok(Arc::new(NetManager::default()) as _)),
        ))
}

impl EngineModule for NetModule {
    fn module_name(&self) -> &'static str {
        NET_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Networking, RPC, and multiplayer sessions"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
