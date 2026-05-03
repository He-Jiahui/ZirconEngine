mod feature;
mod manager;

pub use feature::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetReliableUdpRuntimeFeature, NET_RELIABLE_UDP_FEATURE_CAPABILITY, NET_RELIABLE_UDP_FEATURE_ID,
    NET_RELIABLE_UDP_FEATURE_MANAGER_NAME, NET_RELIABLE_UDP_FEATURE_MODULE_NAME,
};
pub use manager::{net_reliable_udp_runtime_manager, NetReliableUdpRuntimeManager};

#[cfg(test)]
mod tests;
