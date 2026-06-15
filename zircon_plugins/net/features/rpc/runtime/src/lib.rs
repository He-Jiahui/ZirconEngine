mod feature;
mod manager;

pub use feature::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetRpcRuntimeFeature, NET_RPC_FEATURE_CAPABILITY, NET_RPC_FEATURE_ID,
    NET_RPC_FEATURE_MANAGER_NAME, NET_RPC_FEATURE_MODULE_NAME,
};
pub use manager::{net_rpc_runtime_manager, NetRpcRuntimeManager};
pub use manager::{NetRpcHandshakeFrame, RPC_HANDSHAKE_CAPABILITY_NET_RPC, RPC_HANDSHAKE_MAGIC};
pub use manager::{RpcChannelMessage, RPC_CHANNEL_RELIABLE_ORDERED, RPC_CHANNEL_UNRELIABLE};

#[cfg(test)]
mod tests;
