mod capability;
mod feature;
mod manager;
mod packet;
mod plugin;

pub use capability::{NET_RELIABLE_UDP_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use manager::{net_reliable_udp_runtime_manager, NetReliableUdpRuntimeManager};
pub use packet::{
    ReliableUdpFragmentHeader, ReliableUdpWireHeader, ReliableUdpWirePacket,
    ReliableUdpWirePacketError, RELIABLE_UDP_FLAG_FRAGMENT, RELIABLE_UDP_FLAG_LAST_FRAGMENT,
};
pub use plugin::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetReliableUdpRuntimeFeature, NET_RELIABLE_UDP_FEATURE_ID,
    NET_RELIABLE_UDP_FEATURE_MANAGER_NAME, NET_RELIABLE_UDP_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
