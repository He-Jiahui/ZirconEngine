mod capability;
mod feature;
mod manager;
mod plugin;

pub use capability::{NET_REPLICATION_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use manager::{
    net_replication_runtime_manager, NetReplicationRuntimeManager, NetReplicationTable,
    NetReplicationTableEntry,
};
pub use plugin::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetReplicationRuntimeFeature, NET_REPLICATION_FEATURE_ID, NET_REPLICATION_FEATURE_MANAGER_NAME,
    NET_REPLICATION_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
