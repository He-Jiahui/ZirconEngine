mod service_types;

use crate::engine_module::{EngineModule, ModuleDescriptor};

pub use service_types::{NetDriver, NetManager};

pub const NET_MODULE_NAME: &str = "NetModule";
pub const NET_DRIVER_NAME: &str = "NetModule.Driver.NetDriver";
pub const NET_MANAGER_NAME: &str = "NetModule.Manager.NetManager";

#[derive(Clone, Debug, Default)]
pub struct NetConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NetModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<NetDriver, NetManager>(
        NET_MODULE_NAME,
        "Networking, RPC, and multiplayer sessions",
        "NetDriver",
        "NetManager",
    )
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
