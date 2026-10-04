//! 集中公开 WS feature 与 backend 注入入口，供宿主将真实握手/读写接到根 net manager。
//! 无 feature 时根 manager 的 loopback 仍可用，真实网络连接返回能力缺失。

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
