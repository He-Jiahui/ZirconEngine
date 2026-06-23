mod capability;
mod feature;
mod manager;
mod plugin;

pub use capability::{NET_RPC_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use manager::{net_rpc_runtime_manager, NetRpcRuntimeManager};
pub use manager::{NetRpcHandshakeFrame, RPC_HANDSHAKE_CAPABILITY_NET_RPC, RPC_HANDSHAKE_MAGIC};
pub use manager::{RpcChannelMessage, RPC_CHANNEL_RELIABLE_ORDERED, RPC_CHANNEL_UNRELIABLE};
pub use plugin::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetRpcRuntimeFeature, NET_RPC_FEATURE_ID, NET_RPC_FEATURE_MANAGER_NAME,
    NET_RPC_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
