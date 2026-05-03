use std::sync::Arc;

use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};

use crate::net_content_download_runtime_manager;

pub const NET_CONTENT_DOWNLOAD_FEATURE_ID: &str = "net.content_download";
pub const NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY: &str = "runtime.feature.net.cdn_download";
pub const NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME: &str = "NetContentDownloadFeatureModule";
pub const NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME: &str =
    "NetContentDownloadFeatureModule.Manager.NetContentDownloadManager";

#[derive(Clone, Debug)]
pub struct NetContentDownloadRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for NetContentDownloadRuntimeFeature {
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

pub fn runtime_plugin_feature() -> NetContentDownloadRuntimeFeature {
    NetContentDownloadRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
        "Manifest, chunk progress, and cache validation feature for the net plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
            ServiceKind::Manager,
            "NetContentDownloadManager",
        ),
        zircon_runtime::core::StartupMode::Lazy,
        vec![dependency_on(
            zircon_plugin_net_runtime::NET_MODULE_NAME,
            ServiceKind::Manager,
            "NetManager",
        )],
        factory(|_| Ok(Arc::new(net_content_download_runtime_manager()) as ServiceObject)),
    ))
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        NET_CONTENT_DOWNLOAD_FEATURE_ID,
        "Content Download",
        zircon_plugin_net_runtime::PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        zircon_plugin_net_runtime::PLUGIN_ID,
        "runtime.plugin.net",
    ))
    .with_capability(NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY)
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "net.content_download.runtime",
            "zircon_plugin_net_content_download_runtime",
        )
        .with_target_modes([zircon_runtime::RuntimeTargetMode::ClientRuntime])
        .with_capabilities([NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY.to_string()]),
    )
}
