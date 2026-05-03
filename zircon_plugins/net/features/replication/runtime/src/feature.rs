use std::sync::Arc;

use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};

use crate::net_replication_runtime_manager;

pub const NET_REPLICATION_FEATURE_ID: &str = "net.replication";
pub const NET_REPLICATION_FEATURE_CAPABILITY: &str = "runtime.feature.net.replication";
pub const NET_REPLICATION_FEATURE_MODULE_NAME: &str = "NetReplicationFeatureModule";
pub const NET_REPLICATION_FEATURE_MANAGER_NAME: &str =
    "NetReplicationFeatureModule.Manager.NetReplicationManager";

#[derive(Clone, Debug)]
pub struct NetReplicationRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for NetReplicationRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_feature() -> NetReplicationRuntimeFeature {
    NetReplicationRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NET_REPLICATION_FEATURE_MODULE_NAME,
        "Snapshot, delta, and interest filtering feature for the net plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NET_REPLICATION_FEATURE_MODULE_NAME,
            ServiceKind::Manager,
            "NetReplicationManager",
        ),
        zircon_runtime::core::StartupMode::Lazy,
        vec![dependency_on(
            zircon_plugin_net_runtime::NET_MODULE_NAME,
            ServiceKind::Manager,
            "NetManager",
        )],
        factory(|_| Ok(Arc::new(net_replication_runtime_manager()) as ServiceObject)),
    ))
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        NET_REPLICATION_FEATURE_ID,
        "State Replication",
        zircon_plugin_net_runtime::PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        zircon_plugin_net_runtime::PLUGIN_ID,
        "runtime.plugin.net",
    ))
    .with_capability(NET_REPLICATION_FEATURE_CAPABILITY)
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "net.replication.runtime",
            "zircon_plugin_net_replication_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
        ])
        .with_capabilities([NET_REPLICATION_FEATURE_CAPABILITY.to_string()]),
    )
}
