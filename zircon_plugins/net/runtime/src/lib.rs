pub const PLUGIN_ID: &str = "net";

mod config;
mod module;
mod package;
mod service_types;

pub use config::NetConfig;
pub use module::{
    module_descriptor, NetModule, NET_DRIVER_NAME, NET_MANAGER_NAME, NET_MODULE_NAME,
};
pub use package::{
    attach_net_manifest_contributions, net_event_catalogs, net_optional_features, net_options,
    NET_RUNTIME_EVENT_NAMESPACE,
};
pub use service_types::{DefaultNetManager, NetDriver, NetRuntimeManager};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct NetRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl NetRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for NetRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        attach_net_manifest_contributions(self.descriptor.package_manifest())
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        for option in net_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in net_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
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

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        "runtime.plugin.net",
        "runtime.feature.net.http",
        "runtime.feature.net.websocket",
        "runtime.feature.net.rpc",
        "runtime.feature.net.replication",
        "runtime.feature.net.reliable_udp",
        "runtime.feature.net.cdn_download",
    ]
}
