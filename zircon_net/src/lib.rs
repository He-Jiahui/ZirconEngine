//! Networking module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const NET_MODULE_NAME: &str = "NetModule";

#[derive(Clone, Debug, Default)]
pub struct NetConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        NET_MODULE_NAME,
        "Networking, RPC, and multiplayer sessions",
        "NetDriver",
        "NetManager",
    )
}
