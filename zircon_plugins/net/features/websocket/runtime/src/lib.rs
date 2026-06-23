mod backend;
mod capability;
mod feature;
mod plugin;

pub use backend::{websocket_runtime_backend, TungsteniteWebSocketBackend};
pub use capability::{NET_WEBSOCKET_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use plugin::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    websocket_runtime_manager, NetWebSocketRuntimeFeature, NET_WEBSOCKET_FEATURE_ID,
    NET_WEBSOCKET_FEATURE_MANAGER_NAME, NET_WEBSOCKET_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
