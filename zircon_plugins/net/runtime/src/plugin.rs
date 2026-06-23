use crate::capability::{NET_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
use crate::package::{attach_net_manifest_contributions, net_event_catalogs, net_options};
use crate::runtime_system::{
    register_runtime_systems, NET_FLUSH_EGRESS_SYSTEM, NET_POLL_INGRESS_SYSTEM, NET_SYSTEM_SET,
};
use crate::{module_descriptor, PLUGIN_ID};

pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "net.runtime";

#[derive(Clone, Debug)]
pub struct NetRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl NetRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for NetRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        attach_net_manifest_contributions(self.descriptor.package_manifest())
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module(PLUGIN_RUNTIME_MODULE_NAME)?;
        registry.register_module(module_descriptor())?;
        for option in net_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in net_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        register_runtime_systems(registry, owner)?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Network",
        zircon_runtime::builtin::RuntimePluginId::Net,
        "zircon_plugin_net_runtime",
    )
    .with_category("runtime")
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(NET_RUNTIME_CAPABILITY)
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            NET_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
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
