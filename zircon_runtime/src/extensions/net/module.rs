use std::sync::Arc;

use crate::core::manager::NetManagerHandle;
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use super::{DefaultNetManager, NetDriver};

pub const NET_MODULE_NAME: &str = "NetModule";
pub const NET_DRIVER_NAME: &str = "NetModule.Driver.NetDriver";
pub(crate) const DEFAULT_NET_MANAGER_NAME: &str = "NetModule.Manager.DefaultNetManager";
pub const NET_MANAGER_NAME: &str = crate::core::manager::NET_MANAGER_NAME;

#[derive(Clone, Copy, Debug, Default)]
pub struct NetModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(NET_MODULE_NAME, "Networking, RPC, and multiplayer sessions")
        .with_driver(DriverDescriptor::new(
            qualified_name(NET_MODULE_NAME, ServiceKind::Driver, "NetDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(NetDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(NET_MODULE_NAME, ServiceKind::Manager, "DefaultNetManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                NET_MODULE_NAME,
                ServiceKind::Driver,
                "NetDriver",
            )],
            factory(|_| Ok(Arc::new(DefaultNetManager::default()) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(NET_MODULE_NAME, ServiceKind::Manager, "NetManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                NET_MODULE_NAME,
                ServiceKind::Manager,
                "DefaultNetManager",
            )],
            factory(|core| {
                let manager =
                    core.resolve_manager::<DefaultNetManager>(DEFAULT_NET_MANAGER_NAME)?;
                Ok(Arc::new(NetManagerHandle::new(manager)) as ServiceObject)
            }),
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
