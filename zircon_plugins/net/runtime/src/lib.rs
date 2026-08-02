mod capability;
mod config;
mod http;
mod module;
mod package;
mod plugin;
mod poison_recovery;
mod runtime_state;
mod runtime_system;
mod service_types;
mod transport;
mod websocket;
mod worker;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, NET_DECLARATION, NET_RUNTIME_CAPABILITY, PLUGIN_ID,
    RUNTIME_CAPABILITIES,
};
pub use config::NetConfig;
pub use http::{HttpRouteHandler, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute};
pub use module::{
    module_descriptor, NetModule, NET_DRIVER_NAME, NET_MANAGER_NAME, NET_MODULE_NAME,
};
pub use package::{
    attach_net_manifest_contributions, net_event_catalogs, net_optional_features, net_options,
    NET_RUNTIME_EVENT_NAMESPACE,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_package_manifest,
    runtime_plugin, runtime_plugin_descriptor, NetRuntimePlugin, NET_DIST_CRATE_NAME,
    NET_DIST_RUNTIME_ENTRY, PLUGIN_RUNTIME_MODULE_NAME,
};
pub use runtime_system::{
    record_net_diagnostics, register_runtime_systems, NET_DIAGNOSTIC_INBOUND_BYTES,
    NET_DIAGNOSTIC_LAST_LATENCY_MS, NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
    NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS, NET_DIAGNOSTIC_OUTBOUND_BYTES, NET_DIAGNOSTIC_PATHS,
    NET_DIAGNOSTIC_QUEUED_EVENTS, NET_EVENT_ID, NET_EVENT_SCHEMA, NET_FLUSH_EGRESS_SYSTEM,
    NET_MAIN_SYSTEM_SET, NET_POLL_INGRESS_SYSTEM, NET_TRANSPORT_SYSTEM_SET,
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
