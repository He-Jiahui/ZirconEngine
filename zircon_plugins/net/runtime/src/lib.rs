pub const PLUGIN_ID: &str = "net";

mod config;
mod module;
mod service_types;

pub use config::NetConfig;
pub use module::{
    module_descriptor, NetModule, NET_DRIVER_NAME, NET_MANAGER_NAME, NET_MODULE_NAME,
};
pub use service_types::{DefaultNetManager, NetDriver};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct NetRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl NetRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for NetRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::RuntimePluginDescriptor {
    zircon_runtime::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Network",
        zircon_runtime::RuntimePluginId::Net,
        "zircon_plugin_net_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
    ])
    .with_capability("runtime.plugin.net")
}

pub fn runtime_plugin() -> NetRuntimePlugin {
    NetRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::ProjectPluginSelection {
    zircon_runtime::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::RuntimePluginRegistrationReport {
    zircon_runtime::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.net"]
}
