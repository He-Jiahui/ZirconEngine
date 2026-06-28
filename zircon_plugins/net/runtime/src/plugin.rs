use crate::capability::{NET_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
use crate::package::{attach_net_manifest_contributions, net_event_catalogs, net_options};
use crate::runtime_system::{
    register_runtime_systems, NET_FLUSH_EGRESS_SYSTEM, NET_POLL_INGRESS_SYSTEM, NET_SYSTEM_SET,
};
use crate::{module_descriptor, PLUGIN_ID};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "net.runtime";
pub const NET_DIST_CRATE_NAME: &str = "zircon_plugin_net_dist";
pub const NET_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_net_runtime_entry_v3";

const NET_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct NetRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl NetRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for NetRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for NetRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        attach_net_manifest_contributions(runtime_package_manifest())
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
            .module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())?;
        for option in net_options() {
            module.plugin_option(option)?;
        }
        for event_catalog in net_event_catalogs() {
            module.plugin_event_catalog(event_catalog)?;
        }
        register_runtime_systems(&mut module)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Network",
        RuntimePluginId::Net,
        "zircon_plugin_net_runtime",
    )
    .with_category("runtime")
    .with_target_modes([
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(NET_RUNTIME_CAPABILITY)
    .with_maturity(PluginMaturity::Beta)
    .with_capability_status(
        CapabilityStatusManifest::new(NET_RUNTIME_CAPABILITY, CapabilityStatus::Partial)
            .with_bevy_reference("dev/bevy/crates/bevy_remote/src/lib.rs"),
    )
    .with_system_sets([NET_SYSTEM_SET])
    .with_system_anchors([NET_POLL_INGRESS_SYSTEM, NET_FLUSH_EGRESS_SYSTEM])
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(NetRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn runtime_package_manifest() -> PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor().package_manifest();
    manifest
        .default_packaging
        .push(ExportPackagingStrategy::NativeDynamic);
    manifest = manifest.with_native_module(
        PluginModuleManifest::native("net.dist", NET_DIST_CRATE_NAME)
            .with_target_modes([
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
    );
    manifest.with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: NET_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: NET_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: NET_DIST_RUNTIME_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    })
}
