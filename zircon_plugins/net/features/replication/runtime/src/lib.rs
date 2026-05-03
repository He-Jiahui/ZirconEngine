mod feature;
mod manager;

pub use feature::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetReplicationRuntimeFeature, NET_REPLICATION_FEATURE_CAPABILITY, NET_REPLICATION_FEATURE_ID,
    NET_REPLICATION_FEATURE_MANAGER_NAME, NET_REPLICATION_FEATURE_MODULE_NAME,
};
pub use manager::{net_replication_runtime_manager, NetReplicationRuntimeManager};

#[cfg(test)]
mod tests;
