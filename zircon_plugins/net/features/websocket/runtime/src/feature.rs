use std::sync::Arc;

use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};

use crate::websocket_runtime_backend;

pub const NET_WEBSOCKET_FEATURE_ID: &str = "net.websocket";
pub const NET_WEBSOCKET_FEATURE_CAPABILITY: &str = "runtime.feature.net.websocket";
pub const NET_WEBSOCKET_FEATURE_MODULE_NAME: &str = "NetWebSocketFeatureModule";
pub const NET_WEBSOCKET_FEATURE_MANAGER_NAME: &str =
    "NetWebSocketFeatureModule.Manager.NetWebSocketManager";

#[derive(Clone, Debug)]
pub struct NetWebSocketRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for NetWebSocketRuntimeFeature {
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

pub fn runtime_plugin_feature() -> NetWebSocketRuntimeFeature {
    NetWebSocketRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn websocket_runtime_manager() -> zircon_plugin_net_runtime::DefaultNetManager {
    zircon_plugin_net_runtime::DefaultNetManager::default()
        .with_websocket_backend(websocket_runtime_backend())
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NET_WEBSOCKET_FEATURE_MODULE_NAME,
        "WebSocket client and server transport backend for the net plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NET_WEBSOCKET_FEATURE_MODULE_NAME,
            ServiceKind::Manager,
            "NetWebSocketManager",
        ),
        zircon_runtime::core::StartupMode::Lazy,
        vec![dependency_on(
            zircon_plugin_net_runtime::NET_MODULE_NAME,
            ServiceKind::Manager,
            "NetManager",
        )],
        factory(|_| Ok(Arc::new(websocket_runtime_manager()) as ServiceObject)),
    ))
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        NET_WEBSOCKET_FEATURE_ID,
        "WebSocket",
        zircon_plugin_net_runtime::PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        zircon_plugin_net_runtime::PLUGIN_ID,
        "runtime.plugin.net",
    ))
    .with_capability(NET_WEBSOCKET_FEATURE_CAPABILITY)
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "net.websocket.runtime",
            "zircon_plugin_net_websocket_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
        ])
        .with_capabilities([NET_WEBSOCKET_FEATURE_CAPABILITY.to_string()]),
    )
}
