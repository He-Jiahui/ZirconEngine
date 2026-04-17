//! Networking module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const NET_MODULE_NAME: &str = "NetModule";

#[derive(Clone, Debug, Default)]
pub struct NetConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NetModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
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
