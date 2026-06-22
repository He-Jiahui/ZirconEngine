pub const PLUGIN_ID: &str = "net";
pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "net.runtime";

mod capability;
mod config;
mod http;
mod module;
mod package;
mod runtime_state;
mod runtime_system;
mod service_types;
mod transport;
mod websocket;
mod worker;

pub use capability::{NET_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
pub use config::NetConfig;
pub use http::{HttpRouteHandler, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute};
pub use module::{
    module_descriptor, NetModule, NET_DRIVER_NAME, NET_MANAGER_NAME, NET_MODULE_NAME,
};
pub use package::{
    attach_net_manifest_contributions, net_event_catalogs, net_optional_features, net_options,
    NET_RUNTIME_EVENT_NAMESPACE,
};
pub use runtime_system::{
    record_net_diagnostics, register_runtime_systems, NET_DIAGNOSTIC_INBOUND_BYTES,
    NET_DIAGNOSTIC_LAST_LATENCY_MS, NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
    NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS, NET_DIAGNOSTIC_OUTBOUND_BYTES, NET_DIAGNOSTIC_PATHS,
    NET_DIAGNOSTIC_QUEUED_EVENTS, NET_EVENT_ID, NET_EVENT_SCHEMA, NET_FLUSH_EGRESS_SYSTEM,
    NET_POLL_INGRESS_SYSTEM, NET_SYSTEM_SET,
};
pub use service_types::{DefaultNetManager, NetDriver, NetRuntimeManager};
pub use transport::{
    certificate_pin_matches, certificate_sha256_pin, rustls_client_config, rustls_root_store,
    rustls_server_config, TlsServerIdentity,
};
pub use websocket::{
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
};

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

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module(PLUGIN_RUNTIME_MODULE_NAME)?;
        registry.register_module(module_descriptor())?;
        for option in net_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in net_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        register_runtime_systems(registry, owner)?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Network",
        zircon_runtime::builtin::RuntimePluginId::Net,
        "zircon_plugin_net_runtime",
    )
    .with_category("runtime")
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(NET_RUNTIME_CAPABILITY)
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            NET_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_bevy_reference("dev/bevy/crates/bevy_remote/src/lib.rs"),
    )
    .with_system_sets([NET_SYSTEM_SET])
    .with_system_anchors([NET_POLL_INGRESS_SYSTEM, NET_FLUSH_EGRESS_SYSTEM])
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(NetRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
