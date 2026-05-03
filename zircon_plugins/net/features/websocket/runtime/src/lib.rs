mod backend;
mod feature;

pub use backend::{websocket_runtime_backend, TungsteniteWebSocketBackend};
pub use feature::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    websocket_runtime_manager, NetWebSocketRuntimeFeature, NET_WEBSOCKET_FEATURE_CAPABILITY,
    NET_WEBSOCKET_FEATURE_ID, NET_WEBSOCKET_FEATURE_MANAGER_NAME,
    NET_WEBSOCKET_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
